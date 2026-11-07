use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("failed to create the config folder: {0}")]
    CreateFolder(#[source] std::io::Error),

    #[error("failed to read config.ini: {0}")]
    Read(#[source] std::io::Error),

    #[error("failed to write config.ini: {0}")]
    Write(#[source] std::io::Error),

    #[error("theme in config.ini must be light or dark, found {0}")]
    Theme(String),
}
