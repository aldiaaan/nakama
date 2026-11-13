# nakama-daemon

A Tokio daemon runtime. It runs long-lived subsystems, restarts them when they fail, and stops them gracefully on a signal or a request. It can also write a PID file, hold a single-instance lock, record metrics, answer IPC requests, and profile CPU and heap.

The design is based on [proc-daemon](https://github.com/jamesgober/proc-daemon). See `NOTICE`.

## Add it

```toml
[dependencies]
nakama-daemon = { path = "../../crates/nakama-daemon" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The daemon needs a Tokio runtime with time and IO enabled. `#[tokio::main]` and Tauri's async runtime both qualify.

## Run a daemon

Build a `Daemon` from a `Config`, add subsystems, and await `run`. `run` returns once every subsystem has stopped.

```rust
use std::convert::Infallible;
use std::time::Duration;

use nakama_daemon::{Config, Daemon, DaemonError, ShutdownHandle};

#[tokio::main]
async fn main() -> Result<(), DaemonError> {
    Daemon::builder(Config::default())
        .with_task("ticker", |shutdown: ShutdownHandle| async move {
            loop {
                tokio::select! {
                    () = shutdown.cancelled() => return Ok::<_, Infallible>(()),
                    () = tokio::time::sleep(Duration::from_secs(1)) => println!("tick"),
                }
            }
        })
        .build()?
        .run()
        .await
}
```

Press Ctrl+C and the daemon asks every subsystem to stop, then returns.

## Write a subsystem

`with_task` suits a closure that never restarts. Implement `Subsystem` when you want a restart policy, a health check, or state.

```rust
use std::time::Duration;

use nakama_daemon::{RestartPolicy, ShutdownHandle, Subsystem};

#[derive(Debug, thiserror::Error)]
#[error("the queue went away")]
struct QueueGone;

struct Worker;

impl Subsystem for Worker {
    type Error = QueueGone;

    fn name(&self) -> &str {
        "worker"
    }

    async fn run(&self, shutdown: ShutdownHandle) -> Result<(), QueueGone> {
        while !shutdown.is_shutdown() {
            // Do one unit of work, then check for shutdown again.
            tokio::select! {
                () = shutdown.cancelled() => break,
                () = tokio::time::sleep(Duration::from_millis(100)) => {}
            }
        }
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        true
    }

    fn restart_policy(&self) -> RestartPolicy {
        RestartPolicy::ExponentialBackoff {
            initial_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(4),
            max_attempts: 10,
        }
    }
}
```

Add it with `.with_subsystem(Worker)`.

`run` should return soon after `shutdown.cancelled()` resolves. A subsystem still running when the graceful timeout ends is aborted.

| Policy | Restarts after |
|---|---|
| `Never` (default) | Nothing. The subsystem stays stopped or failed. |
| `Always` | Every exit, right away. |
| `OnFailure` | An `Err`, right away. |
| `ExponentialBackoff` | Every exit, waiting `initial_delay` and doubling up to `max_delay`, for up to `max_attempts` restarts. |

A subsystem moves through `SubsystemState`: `Starting`, `Running`, `Restarting`, `Stopping`, then `Stopped`, `Failed`, or `Aborted`.

## Configure it

```rust
use std::time::Duration;

use nakama_daemon::{Config, IpcEndpoint, ShutdownTimeouts};

let config = Config {
    name: "my-daemon".to_owned(),
    shutdown: ShutdownTimeouts {
        graceful: Duration::from_secs(2),
        force: Duration::from_secs(4),
    },
    health_check_interval: Some(Duration::from_secs(5)),
    pid_file: Some(std::env::temp_dir().join("my-daemon.pid")),
    ipc: Some(IpcEndpoint::named("my-daemon")),
};
```

| Field | Default | Meaning |
|---|---|---|
| `name` | `nakama-daemon` | Shown in logs and IPC status. Cannot be empty. |
| `shutdown.graceful` | 5 s | Time subsystems get to return after shutdown starts. Must be above zero. |
| `shutdown.force` | 10 s | Deadline for aborted subsystems to finish. Must be longer than `graceful`, or `run` returns `ShutdownTimeout`. |
| `health_check_interval` | 5 s | How often `Subsystem::is_healthy` is checked. An unhealthy subsystem is logged. `None` turns checks off. |
| `pid_file` | none | Written with the process id while the daemon runs and removed on exit. Its folder must exist. |
| `ipc` | none | Where the IPC channel listens. See [IPC](#ipc). |

`build` validates the config and returns a `DaemonError` for any rule above.

## Signals

By default the daemon shuts down on interrupt, terminate, and quit. Hangup is off.

| `Signal` | Unix | Windows |
|---|---|---|
| `Interrupt` | `SIGINT` | Ctrl+C |
| `Terminate` | `SIGTERM` | Console close or system shutdown |
| `Quit` | `SIGQUIT` | Ctrl+Break |
| `Hangup` | `SIGHUP` | Logoff |

```rust
use nakama_daemon::{Signal, SignalConfig};

builder.with_signals(SignalConfig::default().with(Signal::Hangup).without(Signal::Quit));
```

Call `.without_signals()` when the daemon runs inside another app that owns Ctrl+C, such as a Tauri app.

## Control it from your code

`daemon.handle()` returns a `DaemonHandle`. Clone it freely and take it before calling `run`.

```rust
let daemon = builder.build()?;
let handle = daemon.handle();
let task = tokio::spawn(daemon.run());

handle.is_running();
let stats = handle.stats(); // name, uptime, shutdown reason, and each subsystem's state, restarts, and last error
let metrics = handle.metrics();

handle.shutdown(); // false if shutdown had already started
task.await??;
```

## Metrics

Create a `Metrics`, pass it to the builder with `with_metrics`, and give clones to your subsystems. Everyone records into the same registry.

```rust
use std::time::Duration;

use nakama_daemon::Metrics;

let metrics = Metrics::new();
metrics.increment_counter("jobs_total", 1);
metrics.set_gauge("queue_depth", 12.0);
metrics.record_duration("job", Duration::from_millis(40));
{
    let _timer = metrics.start_timer("job"); // records when dropped
}
```

`handle.metrics()` returns a `MetricsSnapshot` with `counters`, `gauges`, and `timings`. It adds what the daemon tracks itself:

- `daemon_uptime_seconds`
- `daemon_subsystems`, `daemon_subsystems_running`, `daemon_subsystems_failed`
- `daemon_subsystem_restarts_total`, `daemon_subsystem_failures_total`

`snapshot.render_prometheus()` returns Prometheus text format. Timings render as summaries in seconds. Characters Prometheus does not allow in a name become `_`.

## IPC

Set `Config::ipc` and the daemon listens for control requests while it runs:

- On Windows, `IpcEndpoint::named("my-daemon")` is the named pipe `\\.\pipe\my-daemon`.
- Elsewhere it is `my-daemon.sock`, in `$XDG_RUNTIME_DIR` or the temp folder.
- `IpcEndpoint::from_path` takes any path.

Use `IpcClient` from Rust:

```rust
use nakama_daemon::{IpcClient, IpcEndpoint, IpcRequest, IpcResponse};

let mut client = IpcClient::connect(&IpcEndpoint::named("my-daemon")).await?;
if let IpcResponse::Status(status) = client.request(IpcRequest::Status).await? {
    println!("{} has run for {:?} ms", status.name, status.uptime_ms);
}
```

The protocol is one JSON object per line, so any language can talk to it. Each request gets one response line, and a connection can send many requests.

| Request | Response |
|---|---|
| `{"type":"status"}` | `{"type":"status","name":…,"uptimeMs":…,"shutdownReason":…,"subsystems":[{"name","state","restarts","lastError"}]}` |
| `{"type":"metrics"}` | `{"type":"metrics","prometheus":"…"}` |
| `{"type":"shutdown"}` | `{"type":"shutdown","initiated":true}`, or `false` if shutdown had already started |
| `{"type":"call","method":…,"params":…}` | `{"type":"reply","value":…}`, or `{"type":"error","message":…}` if the method failed or is unknown |
| Invalid JSON or an unknown type | `{"type":"error","message":"invalid request: …"}` |

A request over 64 KB gets an error response, and the daemon closes that connection.

From PowerShell:

```powershell
$pipe = New-Object System.IO.Pipes.NamedPipeClientStream('.', 'my-daemon', 'InOut')
$pipe.Connect(2000)
$writer = New-Object System.IO.StreamWriter($pipe); $writer.AutoFlush = $true
$reader = New-Object System.IO.StreamReader($pipe)
$writer.WriteLine('{"type":"status"}'); $reader.ReadLine()
```

The IPC server stops with the daemon and shares its shutdown deadlines.

### Custom methods

Register your own methods on the builder. A handler gets the `DaemonHandle` and the JSON `params`, and returns a future of `Result<serde_json::Value, E>`, where `E` implements `std::error::Error`. An `Err` is sent back as an error response with its message. Registering a name twice keeps the last handler.

```rust
use std::convert::Infallible;

let daemon = Daemon::builder(config)
    .with_ipc_method("double", |_handle, params| async move {
        let n = params["n"].as_u64().unwrap_or_default();
        Ok::<_, Infallible>(serde_json::json!(n * 2))
    })
    .build()?;
```

```rust
let value = client.call("double", serde_json::json!({ "n": 21 })).await?; // 42
```

A failed call returns `DaemonError::IpcMethod` with the handler's message.

## Keep it running after your app exits

A daemon inside your app process dies with it. To outlive the app, run the daemon as its own process and talk to it over IPC. `DetachedDaemon` starts that process unless a daemon already answers on the endpoint:

```rust
use nakama_daemon::{DetachedDaemon, IpcEndpoint, Launch};

let launch = DetachedDaemon::new(IpcEndpoint::named("my-daemon"), std::env::current_exe()?)
    .arg("--daemon")
    .log_file(data_dir.join("daemon.log"))
    .ensure_running()
    .await?;
if let Launch::Started { pid } = launch {
    println!("started daemon {pid}");
}
```

- The program must run a daemon with IPC on at that endpoint. Relaunching your own executable with a flag, as above, is the simplest setup.
- `ensure_running` waits until the endpoint answers, 10 s by default (`start_timeout`). It returns `DetachedExited` if the process quits first and `DetachedStartTimeout` if it never answers.
- If two callers race, the IPC endpoint only lets one daemon bind. The loser exits, and its caller gets `Launch::AlreadyRunning`.
- On Windows the process has no console, gets its own process group, and breaks away from the parent's job object when the job allows it. On Unix it starts a new session with `setsid`.
- Without `log_file`, stdout and stderr are discarded.

Stop it with `{"type":"shutdown"}` over IPC. If the program is your own executable, consider running a copy of it so the original can be rebuilt or updated while the daemon runs.

## Single instance

`InstanceLock` holds an OS file lock until dropped. A second process gets `DaemonError::AlreadyRunning`.

```rust
use nakama_daemon::InstanceLock;

let _lock = InstanceLock::acquire(std::env::temp_dir().join("my-daemon.lock"))?;
```

## Profiling

Both profilers are behind cargo features. Only one of each can run per process.

| Feature | Type | Platforms | Output |
|---|---|---|---|
| `profiling` | `CpuProfiler` | Linux and macOS | pprof protobuf, for `go tool pprof` and similar |
| `heap-profiling` | `HeapProfiler` | All | DHAT JSON, for the [DHAT viewer](https://nnethercote.github.io/dh_view/dh_view.html) |

```rust
// Feature `profiling`, Unix only.
let profiler = nakama_daemon::CpuProfiler::start()?;
// ...
profiler.stop("cpu.pb")?;
```

The heap profiler only sees allocations when the binary installs its allocator:

```rust
#[global_allocator]
static ALLOC: nakama_daemon::HeapProfilerAlloc = nakama_daemon::HeapProfilerAlloc;
```

```rust
// Feature `heap-profiling`.
let profiler = nakama_daemon::HeapProfiler::start("heap.json")?;
let usage = profiler.usage(); // live, peak, and total bytes and blocks
drop(profiler); // writes heap.json
```

That allocator slows every allocation, so keep `heap-profiling` out of release builds.

## Errors

Every fallible call returns `DaemonError`. It covers invalid config, the PID file, signals, the instance lock, shutdown timeouts, IPC, and profiling.

## Example

The Tauri app in `apps/nakama-desktop/src-tauri` runs an example daemon:

- `src/daemon/service.rs` builds it. Four subsystems run calculations on random seeds, share one `Metrics`, and expose `snapshot`, `startProfiler`, and `stopProfiler` IPC methods.
- `src/daemon.rs` controls it. `DaemonMode::Embedded` runs it inside the app and stops it on exit. `DaemonMode::Detached` relaunches a copy of the app executable with `--daemon <data dir>`, and that daemon keeps running after the app closes.
- `DAEMON_MODE` in `src/lib.rs` picks the mode.
