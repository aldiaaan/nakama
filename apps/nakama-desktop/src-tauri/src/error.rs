use std::io;
use std::path::PathBuf;
use std::time::Duration;

use nakama_daemon::DaemonError;
use nakama_device_info::DeviceInfoError;
use nakama_settings::SettingsError;
use serde::Serialize;
use thiserror::Error;

use crate::daemon::{ProfilerKind, Workload};

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum CommandError {
    #[error("{0}")]
    DeviceInfo(String),

    #[error("{0}")]
    Settings(String),

    #[error("{0}")]
    Daemon(String),

    #[error("{0}")]
    Internal(String),
}

/// Starting, querying, and profiling the example daemon.
#[derive(Debug, Error)]
pub enum DaemonControlError {
    #[error(transparent)]
    Daemon(#[from] DaemonError),

    #[error("the daemon is already running")]
    AlreadyRunning,

    #[error("the daemon did not answer over IPC within {0:?}")]
    IpcTimeout(Duration),

    #[error("invalid IPC JSON: {0}")]
    Json(#[source] serde_json::Error),

    #[error("failed to find this executable: {0}")]
    CurrentExe(#[source] io::Error),

    #[error("failed to copy {} to {}: {source}", from.display(), to.display())]
    CopyDaemon {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("the {0} profiler is not available in this build")]
    ProfilerUnavailable(ProfilerKind),

    #[error("the {0} profiler is already running")]
    ProfilerRunning(ProfilerKind),

    #[error("the {0} profiler is not running")]
    ProfilerIdle(ProfilerKind),

    #[error("failed to create the folder at {}: {source}", path.display())]
    CreateFolder {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("the profiler stopped unexpectedly: {0}")]
    Join(#[source] tauri::Error),
}

/// Why an example calculation subsystem exited. The daemon restarts it.
#[derive(Debug, Error)]
pub enum WorkloadError {
    #[error("rejected random input {seed} for {workload}")]
    RejectedInput { workload: Workload, seed: u32 },

    #[error("{workload} calculation stopped unexpectedly: {source}")]
    Join {
        workload: Workload,
        #[source]
        source: tauri::Error,
    },
}

impl From<DeviceInfoError> for CommandError {
    fn from(err: DeviceInfoError) -> Self {
        CommandError::DeviceInfo(err.to_string())
    }
}

impl From<SettingsError> for CommandError {
    fn from(err: SettingsError) -> Self {
        CommandError::Settings(err.to_string())
    }
}

impl From<DaemonControlError> for CommandError {
    fn from(err: DaemonControlError) -> Self {
        CommandError::Daemon(err.to_string())
    }
}
