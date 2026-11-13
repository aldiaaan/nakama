use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::time::Duration;

use thiserror::Error;

use crate::signal::Signal;

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("daemon name cannot be empty")]
    EmptyName,

    #[error("graceful shutdown timeout must be greater than zero")]
    ZeroGracefulTimeout,

    #[error(
        "force shutdown timeout ({force:?}) must be longer than the graceful timeout ({graceful:?})"
    )]
    ForceTimeoutTooShort { graceful: Duration, force: Duration },

    #[error("health check interval must be greater than zero")]
    ZeroHealthCheckInterval,

    #[error("PID file folder does not exist: {}", .0.display())]
    MissingPidFolder(PathBuf),

    #[error("failed to write the PID file at {}: {source}", path.display())]
    WritePidFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to listen for {signal}: {source}")]
    ListenSignal {
        signal: Signal,
        #[source]
        source: io::Error,
    },

    #[error("failed to open the lock file at {}: {source}", path.display())]
    OpenLockFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to lock {}: {source}", path.display())]
    LockFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("another instance holds the lock file at {}", .0.display())]
    AlreadyRunning(PathBuf),

    #[error("subsystems did not stop within {timeout:?}: {}", subsystems.join(", "))]
    ShutdownTimeout {
        timeout: Duration,
        subsystems: Vec<String>,
    },

    #[error("failed to listen for IPC on {}: {source}", endpoint.display())]
    IpcBind {
        endpoint: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to connect to the daemon at {}: {source}", endpoint.display())]
    IpcConnect {
        endpoint: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("IPC connection failed: {0}")]
    Ipc(#[source] io::Error),

    #[error("daemon closed the IPC connection")]
    IpcClosed,

    #[error("daemon sent an invalid IPC response: {0}")]
    IpcResponse(#[source] serde_json::Error),

    #[error("IPC method {method} failed: {message}")]
    IpcMethod { method: String, message: String },

    #[error("daemon answered an IPC call with the wrong response type")]
    UnexpectedIpcResponse,

    #[error("failed to open the daemon log at {}: {source}", path.display())]
    OpenLog {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to start {}: {source}", program.display())]
    SpawnDetached {
        program: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("the detached daemon exited before it answered over IPC ({status})")]
    DetachedExited { status: ExitStatus },

    #[error("the detached daemon did not answer over IPC within {0:?}")]
    DetachedStartTimeout(Duration),

    #[cfg(all(feature = "profiling", unix))]
    #[error("CPU profiler failed: {0}")]
    CpuProfiler(#[source] pprof::Error),

    #[cfg(feature = "heap-profiling")]
    #[error("a heap profiler is already running")]
    HeapProfilerRunning,

    #[cfg(all(feature = "profiling", unix))]
    #[error("failed to write the profile to {}: {source}", path.display())]
    WriteProfile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
