use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceInfoError {
    #[error("failed to read system information: {0}")]
    System(#[from] sysinfo::Error),
}
