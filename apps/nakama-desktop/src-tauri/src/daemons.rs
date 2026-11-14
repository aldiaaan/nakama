mod host;

use tauri::async_runtime;

use crate::error::DaemonControlError;
pub use host::{DaemonHost, DaemonOverview, IpcExchange};

/// Every daemon the app runs, looked up by name.
pub struct Daemons {
    hosts: Vec<DaemonHost>,
}

impl Daemons {
    pub fn new(hosts: impl IntoIterator<Item = DaemonHost>) -> Result<Self, DaemonControlError> {
        let mut unique: Vec<DaemonHost> = Vec::new();
        for host in hosts {
            if unique.iter().any(|existing| existing.name() == host.name()) {
                return Err(DaemonControlError::DuplicateDaemon(host.name().to_owned()));
            }
            unique.push(host);
        }
        Ok(Self { hosts: unique })
    }

    pub fn get(&self, name: &str) -> Result<&DaemonHost, DaemonControlError> {
        self.hosts
            .iter()
            .find(|host| host.name() == name)
            .ok_or_else(|| DaemonControlError::UnknownDaemon(name.to_owned()))
    }

    pub fn start_all(&self) -> Result<(), DaemonControlError> {
        self.hosts.iter().try_for_each(DaemonHost::start)
    }

    /// Shuts every daemon down at once and waits for all of them. Call outside the async runtime, such
    /// as on app exit.
    pub fn stop_all_blocking(&self) {
        let tasks: Vec<_> = self.hosts.iter().filter_map(DaemonHost::shutdown).collect();
        async_runtime::block_on(async {
            for task in tasks {
                let _ = task.await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use nakama_example_daemon::ExampleDaemon;

    use super::{DaemonHost, Daemons};
    use crate::error::DaemonControlError;

    fn host(name: &str) -> DaemonHost {
        let example = ExampleDaemon::new();
        DaemonHost::new(name, move |config| example.builder(config))
    }

    #[test]
    fn looks_daemons_up_by_name() {
        let daemons = Daemons::new([host("first"), host("second")]).expect("unique names");
        assert_eq!(daemons.get("second").expect("second").name(), "second");
        assert!(matches!(
            daemons.get("third"),
            Err(DaemonControlError::UnknownDaemon(name)) if name == "third"
        ));
    }

    #[test]
    fn rejects_a_duplicate_name() {
        assert!(matches!(
            Daemons::new([host("same"), host("same")]),
            Err(DaemonControlError::DuplicateDaemon(name)) if name == "same"
        ));
    }
}
