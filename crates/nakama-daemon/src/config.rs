use std::path::PathBuf;
use std::time::Duration;

use crate::error::DaemonError;
use crate::ipc::IpcEndpoint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub name: String,
    pub shutdown: ShutdownTimeouts,
    /// `None` turns off periodic `Subsystem::is_healthy` checks.
    pub health_check_interval: Option<Duration>,
    /// Written with the process id while the daemon runs, removed on exit.
    pub pid_file: Option<PathBuf>,
    /// Serves status, metrics, and shutdown requests while the daemon runs.
    pub ipc: Option<IpcEndpoint>,
}

/// Both deadlines count from the moment shutdown starts.
///
/// Subsystems get until `graceful` to return. Anything still running is then
/// aborted, and the daemon reports an error if it has not finished by `force`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownTimeouts {
    pub graceful: Duration,
    pub force: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "nakama-daemon".to_owned(),
            shutdown: ShutdownTimeouts::default(),
            health_check_interval: Some(Duration::from_secs(5)),
            pid_file: None,
            ipc: None,
        }
    }
}

impl Default for ShutdownTimeouts {
    fn default() -> Self {
        Self {
            graceful: Duration::from_secs(5),
            force: Duration::from_secs(10),
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<(), DaemonError> {
        if self.name.trim().is_empty() {
            return Err(DaemonError::EmptyName);
        }
        let ShutdownTimeouts { graceful, force } = self.shutdown;
        if graceful.is_zero() {
            return Err(DaemonError::ZeroGracefulTimeout);
        }
        if force <= graceful {
            return Err(DaemonError::ForceTimeoutTooShort { graceful, force });
        }
        if self
            .health_check_interval
            .is_some_and(|interval| interval.is_zero())
        {
            return Err(DaemonError::ZeroHealthCheckInterval);
        }
        if let Some(folder) = self
            .pid_file
            .as_deref()
            .and_then(|path| path.parent())
            .filter(|folder| !folder.as_os_str().is_empty() && !folder.is_dir())
        {
            return Err(DaemonError::MissingPidFolder(folder.to_owned()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, ShutdownTimeouts};
    use crate::error::DaemonError;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn default_config_is_valid() {
        Config::default().validate().expect("valid");
    }

    #[test]
    fn rejects_empty_name() {
        let config = Config {
            name: "  ".to_owned(),
            ..Config::default()
        };
        assert!(matches!(config.validate(), Err(DaemonError::EmptyName)));
    }

    #[test]
    fn rejects_zero_graceful_timeout() {
        let config = Config {
            shutdown: ShutdownTimeouts {
                graceful: Duration::ZERO,
                force: Duration::from_secs(1),
            },
            ..Config::default()
        };
        assert!(matches!(
            config.validate(),
            Err(DaemonError::ZeroGracefulTimeout)
        ));
    }

    #[test]
    fn rejects_force_timeout_not_after_graceful() {
        let config = Config {
            shutdown: ShutdownTimeouts {
                graceful: Duration::from_secs(5),
                force: Duration::from_secs(5),
            },
            ..Config::default()
        };
        assert_eq!(
            config.validate().expect_err("invalid").to_string(),
            "force shutdown timeout (5s) must be longer than the graceful timeout (5s)"
        );
    }

    #[test]
    fn rejects_zero_health_check_interval() {
        let config = Config {
            health_check_interval: Some(Duration::ZERO),
            ..Config::default()
        };
        assert!(matches!(
            config.validate(),
            Err(DaemonError::ZeroHealthCheckInterval)
        ));
    }

    #[test]
    fn accepts_pid_file_in_current_folder() {
        let config = Config {
            pid_file: Some(PathBuf::from("daemon.pid")),
            ..Config::default()
        };
        config.validate().expect("valid");
    }

    #[test]
    fn rejects_missing_pid_folder() {
        let folder = std::env::temp_dir().join("nakama-daemon-missing-pid-folder");
        let config = Config {
            pid_file: Some(folder.join("daemon.pid")),
            ..Config::default()
        };
        assert!(matches!(
            config.validate(),
            Err(DaemonError::MissingPidFolder(path)) if path == folder
        ));
    }
}
