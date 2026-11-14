use std::time::Duration;

use nakama_daemon::{Config, Daemon, DaemonBuilder, Metrics, ShutdownTimeouts};
use serde::Serialize;

use crate::workload::{
    Calculation, CalculationLog, Calculator, JOBS, PI_ESTIMATE, REJECTED, Workload,
};

pub const NAME: &str = "nakama-example-daemon";

/// Four subsystems that run calculations on random seeds. The metrics and the calculation log outlive a
/// single run, so a restarted daemon keeps its history.
#[derive(Debug, Clone, Default)]
pub struct ExampleDaemon {
    metrics: Metrics,
    calculations: CalculationLog,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleReport {
    pub jobs: u64,
    pub rejected_inputs: u64,
    pub pi_estimate: Option<f64>,
    /// Newest first.
    pub calculations: Vec<Calculation>,
}

impl ExampleDaemon {
    pub fn new() -> Self {
        Self::default()
    }

    /// Keeps the name, IPC endpoint, and PID file from `config`, and sets the shutdown timeouts.
    pub fn builder(&self, config: Config) -> DaemonBuilder {
        let config = Config {
            shutdown: ShutdownTimeouts {
                graceful: Duration::from_secs(2),
                force: Duration::from_secs(4),
            },
            ..config
        };
        Workload::ALL.into_iter().fold(
            Daemon::builder(config).with_metrics(self.metrics.clone()),
            |builder, workload| {
                builder.with_subsystem(Calculator::new(
                    workload,
                    self.metrics.clone(),
                    self.calculations.clone(),
                ))
            },
        )
    }

    pub fn report(&self) -> ExampleReport {
        let snapshot = self.metrics.snapshot();
        ExampleReport {
            jobs: snapshot.counters.get(JOBS).copied().unwrap_or_default(),
            rejected_inputs: snapshot.counters.get(REJECTED).copied().unwrap_or_default(),
            pi_estimate: snapshot.gauges.get(PI_ESTIMATE).copied(),
            calculations: self.calculations.recent(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use nakama_daemon::Config;
    use tokio::runtime::Runtime;
    use tokio::time;

    use super::ExampleDaemon;

    #[test]
    fn records_calculations_across_runs() {
        let runtime = Runtime::new().expect("runtime");
        let example = ExampleDaemon::new();

        let run_until_first_result = |minimum: usize| {
            let daemon = example
                .builder(Config::default())
                .without_signals()
                .build()
                .expect("build");
            assert_eq!(daemon.handle().stats().subsystems.len(), 4);
            let handle = daemon.handle();
            runtime.block_on(async {
                let run = tokio::spawn(daemon.run());
                time::timeout(Duration::from_secs(10), async {
                    while example.report().calculations.len() < minimum {
                        time::sleep(Duration::from_millis(20)).await;
                    }
                })
                .await
                .expect("a new calculation");
                handle.shutdown();
                run.await.expect("join").expect("run");
            });
        };

        run_until_first_result(1);
        let first = example.report();
        assert!(first.jobs >= 1);
        run_until_first_result(first.calculations.len() + 1);
        assert!(example.report().jobs > first.jobs);
    }
}
