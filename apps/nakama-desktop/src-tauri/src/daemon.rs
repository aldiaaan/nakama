mod profilers;
mod workload;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use nakama_daemon::{
    Config, Daemon, DaemonError, DaemonHandle, IpcClient, IpcEndpoint, IpcRequest, IpcResponse,
    Metrics, ShutdownTimeouts, StatusReport,
};
use serde::Serialize;
use tauri::async_runtime::{self, JoinHandle};
use tokio::time;

use crate::error::DaemonControlError;
pub use profilers::ProfilerKind;
use profilers::{ProfilerReport, Profilers};
pub use workload::Workload;
use workload::{Calculation, CalculationLog, Calculator};

pub const NAME: &str = "nakama-daemon";
const IPC_TIMEOUT: Duration = Duration::from_secs(1);

/// Owns the example daemon that runs in the background for as long as the app is open.
pub struct DaemonController {
    endpoint: IpcEndpoint,
    profiles_dir: PathBuf,
    calculations: CalculationLog,
    run: Mutex<Option<Run>>,
    profilers: Mutex<Profilers>,
}

struct Run {
    handle: DaemonHandle,
    result: Arc<Mutex<Option<Result<(), DaemonError>>>>,
    task: JoinHandle<()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DaemonState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonOverview {
    pub state: DaemonState,
    pub error: Option<String>,
    pub status: Option<StatusReport>,
    pub metrics: MetricsReport,
    pub prometheus: String,
    pub ipc: IpcReport,
    pub calculations: Vec<Calculation>,
    pub profilers: Vec<ProfilerReport>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsReport {
    pub counters: BTreeMap<String, u64>,
    pub gauges: BTreeMap<String, f64>,
    pub timings: BTreeMap<String, TimingReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimingReport {
    pub count: u64,
    pub total_ms: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcReport {
    pub endpoint: String,
    pub reachable: bool,
    pub round_trip_us: Option<u64>,
    pub error: Option<String>,
}

/// One request sent over the daemon's IPC channel, as the JSON lines on the wire.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcExchange {
    pub request: String,
    pub response: String,
    pub round_trip_us: u64,
}

impl DaemonController {
    pub fn new(endpoint: IpcEndpoint, profiles_dir: PathBuf) -> Self {
        Self {
            endpoint,
            profiles_dir,
            calculations: CalculationLog::default(),
            run: Mutex::new(None),
            profilers: Mutex::new(Profilers::default()),
        }
    }

    pub fn start(&self) -> Result<(), DaemonControlError> {
        let mut run = lock(&self.run);
        if run.as_ref().is_some_and(|run| lock(&run.result).is_none()) {
            return Err(DaemonControlError::AlreadyRunning);
        }

        let metrics = Metrics::new();
        let builder = Workload::ALL.into_iter().fold(
            Daemon::builder(Config {
                name: NAME.to_owned(),
                shutdown: ShutdownTimeouts {
                    graceful: Duration::from_secs(2),
                    force: Duration::from_secs(4),
                },
                ipc: Some(self.endpoint.clone()),
                ..Config::default()
            })
            // Ctrl+C belongs to the app, not to the daemon inside it.
            .without_signals()
            .with_metrics(metrics.clone()),
            |builder, workload| {
                builder.with_subsystem(Calculator::new(
                    workload,
                    metrics.clone(),
                    self.calculations.clone(),
                ))
            },
        );
        let daemon = builder.build()?;
        let handle = daemon.handle();
        let result = Arc::new(Mutex::new(None));
        let task = async_runtime::spawn({
            let result = Arc::clone(&result);
            async move {
                let outcome = daemon.run().await;
                if let Err(error) = &outcome {
                    eprintln!("daemon stopped: {error}");
                }
                *lock(&result) = Some(outcome);
            }
        });
        *run = Some(Run {
            handle,
            result,
            task,
        });
        Ok(())
    }

    /// Stops the daemon and waits for it. Call outside the async runtime, such as on app exit.
    pub fn stop_blocking(&self) {
        let Some(run) = lock(&self.run).take() else {
            return;
        };
        run.handle.shutdown();
        let _ = async_runtime::block_on(run.task);
    }

    pub async fn overview(&self) -> DaemonOverview {
        let ipc = self.ping().await;
        let (state, error, status, snapshot) = {
            let run = lock(&self.run);
            match run.as_ref() {
                None => (DaemonState::Stopped, None, None, None),
                Some(run) => {
                    let (state, error) = match lock(&run.result).as_ref() {
                        Some(Ok(())) => (DaemonState::Stopped, None),
                        Some(Err(error)) => (DaemonState::Failed, Some(error.to_string())),
                        None if run.handle.is_running() => (DaemonState::Running, None),
                        None if run.handle.stats().shutdown_reason.is_some() => {
                            (DaemonState::Stopping, None)
                        }
                        None => (DaemonState::Starting, None),
                    };
                    let status = StatusReport::from(&run.handle.stats());
                    (state, error, Some(status), Some(run.handle.metrics()))
                }
            }
        };
        let snapshot = snapshot.unwrap_or_default();

        DaemonOverview {
            state,
            error,
            status,
            prometheus: snapshot.render_prometheus(),
            metrics: MetricsReport {
                counters: snapshot.counters,
                gauges: snapshot.gauges,
                timings: snapshot
                    .timings
                    .into_iter()
                    .map(|(name, timing)| {
                        let report = TimingReport {
                            count: timing.count,
                            total_ms: timing.total.as_secs_f64() * 1000.0,
                        };
                        (name, report)
                    })
                    .collect(),
            },
            ipc,
            calculations: self.calculations.recent(),
            profilers: lock(&self.profilers).reports(),
        }
    }

    pub async fn send_ipc(&self, request: IpcRequest) -> Result<IpcExchange, DaemonControlError> {
        let started = Instant::now();
        let response = self.request(request).await?;
        Ok(IpcExchange {
            request: serde_json::to_string(&request).map_err(DaemonControlError::Encode)?,
            response: serde_json::to_string(&response).map_err(DaemonControlError::Encode)?,
            round_trip_us: micros(started.elapsed()),
        })
    }

    pub fn start_profiler(&self, kind: ProfilerKind) -> Result<(), DaemonControlError> {
        lock(&self.profilers).start(kind, &self.profiles_dir)
    }

    pub async fn stop_profiler(&self, kind: ProfilerKind) -> Result<(), DaemonControlError> {
        let finish = lock(&self.profilers).stop(kind)?;
        let path = async_runtime::spawn_blocking(finish)
            .await
            .map_err(DaemonControlError::Join)??;
        lock(&self.profilers).finished(kind, path);
        Ok(())
    }

    async fn ping(&self) -> IpcReport {
        let started = Instant::now();
        let result = self.request(IpcRequest::Status).await;
        IpcReport {
            endpoint: self.endpoint.to_string(),
            reachable: result.is_ok(),
            round_trip_us: result.is_ok().then(|| micros(started.elapsed())),
            error: result.err().map(|error| error.to_string()),
        }
    }

    async fn request(&self, request: IpcRequest) -> Result<IpcResponse, DaemonControlError> {
        let exchange = async {
            let mut client = IpcClient::connect(&self.endpoint).await?;
            client.request(request).await
        };
        Ok(time::timeout(IPC_TIMEOUT, exchange)
            .await
            .map_err(|_| DaemonControlError::IpcTimeout(IPC_TIMEOUT))??)
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::process;
    use std::time::Duration;

    use nakama_daemon::{IpcEndpoint, IpcRequest};
    use tauri::async_runtime;
    use tokio::time;

    use super::{DaemonController, DaemonOverview, DaemonState};
    use crate::error::DaemonControlError;

    async fn wait_for(
        controller: &DaemonController,
        ready: impl Fn(&DaemonOverview) -> bool,
    ) -> DaemonOverview {
        for _ in 0..200 {
            let overview = controller.overview().await;
            if ready(&overview) {
                return overview;
            }
            time::sleep(Duration::from_millis(50)).await;
        }
        panic!("the daemon never reached the expected state");
    }

    #[test]
    fn runs_answers_ipc_and_starts_again() {
        let name = format!("nakama-desktop-test-{}", process::id());
        let controller =
            DaemonController::new(IpcEndpoint::named(&name), std::env::temp_dir().join(&name));
        controller.start().expect("start");
        assert!(matches!(
            controller.start(),
            Err(DaemonControlError::AlreadyRunning)
        ));

        async_runtime::block_on(async {
            let overview = wait_for(&controller, |overview| {
                overview.ipc.reachable && !overview.calculations.is_empty()
            })
            .await;
            assert_eq!(overview.state, DaemonState::Running);
            assert_eq!(
                overview.status.map(|status| status.subsystems.len()),
                Some(4)
            );
            assert!(overview.metrics.counters["workload_jobs_total"] >= 1);
            assert!(
                overview
                    .prometheus
                    .contains("# TYPE workload_jobs_total counter")
            );

            let exchange = controller
                .send_ipc(IpcRequest::Shutdown)
                .await
                .expect("shutdown over IPC");
            assert_eq!(exchange.request, r#"{"type":"shutdown"}"#);
            assert_eq!(exchange.response, r#"{"type":"shutdown","initiated":true}"#);

            let overview = wait_for(&controller, |overview| {
                overview.state == DaemonState::Stopped
            })
            .await;
            assert!(!overview.ipc.reachable);
        });

        controller.start().expect("start again");
        controller.stop_blocking();
    }

    #[cfg(feature = "heap-profiling")]
    #[test]
    fn writes_a_heap_profile() {
        use super::ProfilerKind;

        let dir = std::env::temp_dir().join(format!("nakama-desktop-heap-{}", process::id()));
        let controller = DaemonController::new(IpcEndpoint::named("unused"), dir.clone());
        controller
            .start_profiler(ProfilerKind::Heap)
            .expect("start heap profiler");
        assert!(matches!(
            controller.start_profiler(ProfilerKind::Heap),
            Err(DaemonControlError::ProfilerRunning(ProfilerKind::Heap))
        ));
        async_runtime::block_on(controller.stop_profiler(ProfilerKind::Heap))
            .expect("stop heap profiler");

        let reports = super::lock(&controller.profilers).reports();
        let profile = reports[1].last_profile.as_deref().expect("profile path");
        assert!(std::path::Path::new(profile).is_file());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[cfg(not(all(feature = "profiling", unix)))]
    #[test]
    fn reports_a_missing_cpu_profiler() {
        use super::ProfilerKind;

        let controller = DaemonController::new(IpcEndpoint::named("unused"), std::env::temp_dir());
        assert!(matches!(
            controller.start_profiler(ProfilerKind::Cpu),
            Err(DaemonControlError::ProfilerUnavailable(ProfilerKind::Cpu))
        ));
    }
}
