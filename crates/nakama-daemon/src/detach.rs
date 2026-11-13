use std::ffi::OsString;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use tokio::time::{self, Instant};

use crate::error::DaemonError;
use crate::ipc::{IpcClient, IpcEndpoint};

const POLL: Duration = Duration::from_millis(50);

/// Starts a daemon program in its own process, so it keeps running after the caller exits.
///
/// The program should run a daemon that listens on `endpoint`. An answer on that endpoint
/// counts as running, so `ensure_running` starts at most one copy.
#[derive(Debug, Clone)]
pub struct DetachedDaemon {
    endpoint: IpcEndpoint,
    program: PathBuf,
    args: Vec<OsString>,
    log: Option<PathBuf>,
    start_timeout: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Launch {
    AlreadyRunning,
    Started { pid: u32 },
}

impl DetachedDaemon {
    pub fn new(endpoint: IpcEndpoint, program: impl Into<PathBuf>) -> Self {
        Self {
            endpoint,
            program: program.into(),
            args: Vec::new(),
            log: None,
            start_timeout: Duration::from_secs(10),
        }
    }

    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Appends the program's stdout and stderr to `path`. Without a log, output is discarded.
    pub fn log_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.log = Some(path.into());
        self
    }

    /// How long `ensure_running` waits for a new process to answer over IPC. Defaults to 10 s.
    pub fn start_timeout(mut self, timeout: Duration) -> Self {
        self.start_timeout = timeout;
        self
    }

    pub async fn is_running(&self) -> bool {
        IpcClient::connect(&self.endpoint).await.is_ok()
    }

    /// Starts the program unless a daemon already answers, then waits until it does.
    pub async fn ensure_running(&self) -> Result<Launch, DaemonError> {
        if self.is_running().await {
            return Ok(Launch::AlreadyRunning);
        }
        let mut child = self.spawn()?;
        let pid = child.id();
        let deadline = Instant::now() + self.start_timeout;
        loop {
            let running = self.is_running().await;
            match child.try_wait() {
                // Another launcher won the race, and this copy could not take the endpoint.
                Ok(Some(_)) if running => return Ok(Launch::AlreadyRunning),
                Ok(Some(status)) => return Err(DaemonError::DetachedExited { status }),
                _ if running => return Ok(Launch::Started { pid }),
                _ => {}
            }
            if Instant::now() >= deadline {
                return Err(DaemonError::DetachedStartTimeout(self.start_timeout));
            }
            time::sleep(POLL).await;
        }
    }

    fn spawn(&self) -> Result<Child, DaemonError> {
        let mut command = Command::new(&self.program);
        command.args(&self.args).stdin(Stdio::null());
        match &self.log {
            Some(path) => {
                let open_log = |source| DaemonError::OpenLog {
                    path: path.clone(),
                    source,
                };
                let file = File::options()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(open_log)?;
                command
                    .stdout(file.try_clone().map_err(open_log)?)
                    .stderr(file);
            }
            None => {
                command.stdout(Stdio::null()).stderr(Stdio::null());
            }
        }
        platform::spawn_detached(&mut command).map_err(|source| DaemonError::SpawnDetached {
            program: self.program.clone(),
            source,
        })
    }
}

#[cfg(windows)]
mod platform {
    use std::io;
    use std::os::windows::process::CommandExt;
    use std::process::{Child, Command};

    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
    const ERROR_ACCESS_DENIED: i32 = 5;

    /// Leaves the parent's console and process group, and its job object when the job allows
    /// it. A job that kills its processes on close, such as `cargo run`'s, would end the daemon
    /// with the parent.
    pub(super) fn spawn_detached(command: &mut Command) -> io::Result<Child> {
        let flags = DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP;
        match command
            .creation_flags(flags | CREATE_BREAKAWAY_FROM_JOB)
            .spawn()
        {
            Err(error) if error.raw_os_error() == Some(ERROR_ACCESS_DENIED) => {
                command.creation_flags(flags).spawn()
            }
            spawned => spawned,
        }
    }
}

#[cfg(unix)]
mod platform {
    use std::io;
    use std::os::unix::process::CommandExt;
    use std::process::{Child, Command};

    /// Starts a new session, so closing the parent's terminal does not hang up the daemon.
    pub(super) fn spawn_detached(command: &mut Command) -> io::Result<Child> {
        // SAFETY: setsid is async-signal-safe, so it may run between fork and exec.
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        command.spawn()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::process;
    use std::time::Duration;

    use super::{DetachedDaemon, Launch};
    use crate::config::Config;
    use crate::daemon::Daemon;
    use crate::error::DaemonError;
    use crate::ipc::IpcEndpoint;
    use crate::shutdown::ShutdownHandle;

    fn endpoint(test: &str) -> IpcEndpoint {
        IpcEndpoint::named(&format!("nakama-daemon-detach-{test}-{}", process::id()))
    }

    #[tokio::test]
    async fn reports_a_program_that_cannot_start() {
        let launcher = DetachedDaemon::new(endpoint("missing"), "nakama-daemon-no-such-program");
        assert!(matches!(
            launcher.ensure_running().await,
            Err(DaemonError::SpawnDetached { .. })
        ));
    }

    #[tokio::test]
    async fn reports_a_program_that_exits_early() {
        let launcher = if cfg!(windows) {
            DetachedDaemon::new(endpoint("exits"), "cmd")
                .arg("/C")
                .arg("exit 3")
        } else {
            DetachedDaemon::new(endpoint("exits"), "sh")
                .arg("-c")
                .arg("exit 3")
        };
        assert!(matches!(
            launcher.ensure_running().await,
            Err(DaemonError::DetachedExited { status }) if status.code() == Some(3)
        ));
    }

    #[tokio::test]
    async fn does_not_start_a_second_copy() {
        let endpoint = endpoint("running");
        let daemon = Daemon::builder(Config {
            ipc: Some(endpoint.clone()),
            health_check_interval: None,
            ..Config::default()
        })
        .without_signals()
        .with_task("idle", |shutdown: ShutdownHandle| async move {
            shutdown.cancelled().await;
            Ok::<_, Infallible>(())
        })
        .build()
        .expect("build");
        let handle = daemon.handle();
        let run = tokio::spawn(daemon.run());

        let launcher = DetachedDaemon::new(endpoint, "nakama-daemon-no-such-program")
            .start_timeout(Duration::from_secs(1));
        for _ in 0..100 {
            if launcher.is_running().await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(
            launcher.ensure_running().await.expect("launch"),
            Launch::AlreadyRunning
        );

        handle.shutdown();
        run.await.expect("join").expect("run");
    }
}
