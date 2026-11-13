use std::fs::{File, TryLockError};
use std::path::Path;

use crate::error::DaemonError;

/// Keeps one daemon instance per lock file. The lock is released on drop.
#[derive(Debug)]
pub struct InstanceLock {
    _file: File,
}

impl InstanceLock {
    pub fn acquire(path: impl AsRef<Path>) -> Result<Self, DaemonError> {
        let path = path.as_ref();
        let file = File::options()
            .create(true)
            .write(true)
            .truncate(false)
            .open(path)
            .map_err(|source| DaemonError::OpenLockFile {
                path: path.to_owned(),
                source,
            })?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(DaemonError::AlreadyRunning(path.to_owned())),
            Err(TryLockError::Error(source)) => Err(DaemonError::LockFile {
                path: path.to_owned(),
                source,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InstanceLock;
    use crate::error::DaemonError;
    use std::{fs, process};

    #[test]
    fn allows_one_holder_at_a_time() {
        let path = std::env::temp_dir().join(format!("nakama-daemon-lock-{}", process::id()));
        let first = InstanceLock::acquire(&path).expect("first lock");
        assert!(matches!(
            InstanceLock::acquire(&path),
            Err(DaemonError::AlreadyRunning(locked)) if locked == path
        ));
        drop(first);
        let second = InstanceLock::acquire(&path).expect("lock after release");
        drop(second);
        let _ = fs::remove_file(path);
    }
}
