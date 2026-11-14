use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use nakama_daemon::{
    Config, DaemonBuilder, DaemonError, DaemonHandle, IpcClient, IpcEndpoint, IpcRequest,
    IpcResponse, StatusReport,
};
use serde::Serialize;
use tauri::async_runtime::{self, JoinHandle};
use tokio::time;

use crate::error::DaemonControlError;

const IPC_TIMEOUT: Duration = Duration::from_secs(1);

type BuildDaemon = Box<dyn Fn(Config) -> DaemonBuilder + Send + Sync>;

/// Runs one daemon inside the app process, on the Tauri async runtime.
pub struct DaemonHost {
    name: String,
    endpoint: IpcEndpoint,
    build: BuildDaemon,
    run: Mutex<Option<Run>>,
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
    pub name: String,
    pub state: DaemonState,
    pub error: Option<String>,
    pub status: Option<StatusReport>,
    pub metrics: MetricsReport,
    pub prometheus: String,
    pub ipc: IpcReport,
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

impl DaemonHost {
    /// `build` adds the daemon's subsystems to a builder made from the config the host passes in, which
    /// already holds the name and the IPC endpoint. The IPC endpoint is named after the daemon.
    pub fn new(
        name: impl Into<String>,
        build: impl Fn(Config) -> DaemonBuilder + Send + Sync + 'static,
    ) -> Self {
        let name = name.into();
        Self {
            endpoint: IpcEndpoint::named(&name),
            name,
            build: Box::new(build),
            run: Mutex::new(None),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn start(&self) -> Result<(), DaemonControlError> {
        let mut run = lock(&self.run);
        if run.as_ref().is_some_and(|run| lock(&run.result).is_none()) {
            return Err(DaemonControlError::AlreadyRunning(self.name.clone()));
        }

        let daemon = (self.build)(Config {
            name: self.name.clone(),
            ipc: Some(self.endpoint.clone()),
            ..Config::default()
        })
        // Ctrl+C belongs to the app, not to a daemon inside it.
        .without_signals()
        .build()?;
        let handle = daemon.handle();
        let result = Arc::new(Mutex::new(None));
        let task = async_runtime::spawn({
            let result = Arc::clone(&result);
            let name = self.name.clone();
            async move {
                let outcome = daemon.run().await;
                if let Err(error) = &outcome {
                    eprintln!("{name} stopped: {error}");
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

    /// Starts shutting the daemon down. Await the task to wait for it.
    pub fn shutdown(&self) -> Option<JoinHandle<()>> {
        let run = lock(&self.run).take()?;
        run.handle.shutdown();
        Some(run.task)
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
            name: self.name.clone(),
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

    use nakama_daemon::IpcRequest;
    use nakama_example_daemon::ExampleDaemon;
    use tauri::async_runtime;
    use tokio::time;

    use super::{DaemonHost, DaemonOverview, DaemonState};
    use crate::error::DaemonControlError;

    async fn wait_for(
        host: &DaemonHost,
        ready: impl Fn(&DaemonOverview) -> bool,
    ) -> DaemonOverview {
        for _ in 0..200 {
            let overview = host.overview().await;
            if ready(&overview) {
                return overview;
            }
            time::sleep(Duration::from_millis(50)).await;
        }
        panic!("the daemon never reached the expected state");
    }

    #[test]
    fn runs_answers_ipc_and_starts_again() {
        let example = ExampleDaemon::new();
        let host = DaemonHost::new(
            format!("nakama-desktop-test-{}", process::id()),
            move |config| example.builder(config),
        );
        host.start().expect("start");
        assert!(matches!(
            host.start(),
            Err(DaemonControlError::AlreadyRunning(_))
        ));

        async_runtime::block_on(async {
            let overview = wait_for(&host, |overview| {
                overview.ipc.reachable
                    && overview
                        .metrics
                        .counters
                        .contains_key("workload_jobs_total")
            })
            .await;
            assert_eq!(overview.name, host.name());
            assert_eq!(overview.state, DaemonState::Running);
            assert_eq!(
                overview.status.map(|status| status.subsystems.len()),
                Some(4)
            );
            assert!(
                overview
                    .prometheus
                    .contains("# TYPE workload_jobs_total counter")
            );

            let exchange = host
                .send_ipc(IpcRequest::Shutdown)
                .await
                .expect("shutdown over IPC");
            assert_eq!(exchange.request, r#"{"type":"shutdown"}"#);
            assert_eq!(exchange.response, r#"{"type":"shutdown","initiated":true}"#);

            let overview = wait_for(&host, |overview| overview.state == DaemonState::Stopped).await;
            assert!(!overview.ipc.reachable);
        });

        host.start().expect("start again");
        let task = host.shutdown().expect("a running daemon");
        async_runtime::block_on(task).expect("join");
        assert_eq!(
            async_runtime::block_on(host.overview()).state,
            DaemonState::Stopped
        );
    }
}
