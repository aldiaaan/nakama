use std::io;
use std::path::PathBuf;
use std::time::Duration;

use nakama_daemon::DaemonError;
use nakama_device_info::DeviceInfoError;
use nakama_settings::SettingsError;
use serde::Serialize;
use thiserror::Error;

use crate::profilers::ProfilerKind;

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
    Profiler(String),

    #[error("{0}")]
    Notification(String),

    #[error("{0}")]
    Internal(String),
}

/// Starting and querying the daemons the app runs.
#[derive(Debug, Error)]
pub enum DaemonControlError {
    #[error(transparent)]
    Daemon(#[from] DaemonError),

    #[error("there is no daemon named {0}")]
    UnknownDaemon(String),

    #[error("more than one daemon is named {0}")]
    DuplicateDaemon(String),

    #[error("the {0} daemon is already running")]
    AlreadyRunning(String),

    #[error("the daemon did not answer over IPC within {0:?}")]
    IpcTimeout(Duration),

    #[error("failed to encode an IPC message: {0}")]
    Encode(#[source] serde_json::Error),
}

/// Showing a notification in the operating system's notification center.
#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("a notification needs a title")]
    EmptyTitle,

    #[error("a notification needs a body")]
    EmptyBody,

    #[error("notification permission was denied")]
    PermissionDenied,

    #[error("failed to show the notification: {0}")]
    Show(#[from] tauri_plugin_notification::Error),
}

/// Starting and stopping the app's CPU and heap profilers.
#[derive(Debug, Error)]
pub enum ProfilerError {
    #[error(transparent)]
    Daemon(#[from] DaemonError),

    #[error("the {0} profiler is not available in this build")]
    Unavailable(ProfilerKind),

    #[error("the {0} profiler is already running")]
    Running(ProfilerKind),

    #[error("the {0} profiler is not running")]
    Idle(ProfilerKind),

    #[error("failed to create the profiles folder at {}: {source}", path.display())]
    ProfilesFolder {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("the profiler stopped unexpectedly: {0}")]
    Join(#[source] tauri::Error),
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

impl From<ProfilerError> for CommandError {
    fn from(err: ProfilerError) -> Self {
        CommandError::Profiler(err.to_string())
    }
}

impl From<NotificationError> for CommandError {
    fn from(err: NotificationError) -> Self {
        CommandError::Notification(err.to_string())
    }
}
