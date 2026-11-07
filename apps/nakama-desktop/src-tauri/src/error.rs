use nakama_device_info::DeviceInfoError;
use nakama_settings::SettingsError;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum CommandError {
    #[error("{0}")]
    DeviceInfo(String),

    #[error("{0}")]
    Settings(String),

    #[error("{0}")]
    Internal(String),
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
