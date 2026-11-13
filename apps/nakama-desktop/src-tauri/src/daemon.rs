mod profilers;
mod service;
mod workload;

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use nakama_daemon::{
    DaemonError, DaemonHandle, DetachedDaemon, IpcClient, IpcEndpoint, IpcRequest, IpcResponse,
    SignalConfig, StatusReport,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tauri::async_runtime::{self, JoinHandle};
use tokio::time;

use crate::error::DaemonControlError;
pub use profilers::ProfilerKind;
use profilers::ProfilerReport;
use service::{MetricsReport, ProfilerParams, Snapshot};
pub use service::{NAME, run_standalone};
use workload::Calculation;
pub use workload::Workload;

/// Passed to this executable, with the data folder, to run only the daemon.
pub const DAEMON_FLAG: &str = "--daemon";

const IPC_TIMEOUT: Duration = Duration::from_secs(1);
const PROFILE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DaemonMode {
    /// Runs on the app's async runtime and stops when the app exits.
    Embedded,
    /// Runs in a separate copy of this executable and keeps running after the app exits.
    Detached,
}

/// Starts the example daemon and reads it over IPC, whichever process it runs in.
pub struct DaemonController {
    mode: DaemonMode,
    endpoint: IpcEndpoint,
    data_dir: PathBuf,
    embedded: Mutex<Option<Run>>,
    /// Held while a detached launch is in flight. Keeps the last launch error.
    launch: tokio::sync::Mutex<Option<String>>,
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
    pub mode: DaemonMode,
    pub state: DaemonState,
    pub error: Option<String>,
    pub pid: Option<u32>,
    pub status: Option<StatusReport>,
    pub metrics: MetricsReport,
    pub prometheus: String,
    pub ipc: IpcReport,
    pub calculations: Vec<Calculation>,
    pub profilers: Vec<ProfilerReport>,
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
    pub fn new(mode: DaemonMode, endpoint: IpcEndpoint, data_dir: PathBuf) -> Self {
        Self {
            mode,
            endpoint,
            data_dir,
            embedded: Mutex::new(None),
            launch: tokio::sync::Mutex::new(None),
        }
    }

    pub async fn start(&self) -> Result<(), DaemonControlError> {
        match self.mode {
            DaemonMode::Embedded => self.start_embedded(),
            DaemonMode::Detached => {
                let mut last_error = self.launch.lock().await;
                let launched = self.launch_detached().await;
                *last_error = launched.as_ref().err().map(ToString::to_string);
                launched
            }
        }
    }

    /// Stops an embedded daemon and waits for it. A detached daemon keeps running.
    ///
    /// Call outside the async runtime, such as on app exit.
    pub fn stop_blocking(&self) {
        let Some(run) = lock(&self.embedded).take() else {
            return;
        };
        run.handle.shutdown();
        let _ = async_runtime::block_on(run.task);
    }

    pub async fn overview(&self) -> DaemonOverview {
        let started = Instant::now();
        let snapshot = self
            .call::<Snapshot>(service::SNAPSHOT, Value::Null, IPC_TIMEOUT)
            .await;
        let round_trip_us = micros(started.elapsed());
        let endpoint = self.endpoint.to_string();

        match snapshot {
            Ok(snapshot) => DaemonOverview {
                mode: self.mode,
                state: if snapshot.status.shutdown_reason.is_some() {
                    DaemonState::Stopping
                } else {
                    DaemonState::Running
                },
                error: None,
                pid: Some(snapshot.pid),
                status: Some(snapshot.status),
                metrics: snapshot.metrics,
                prometheus: snapshot.prometheus,
                ipc: IpcReport {
                    endpoint,
                    reachable: true,
                    round_trip_us: Some(round_trip_us),
                    error: None,
                },
                calculations: snapshot.calculations,
                profilers: snapshot.profilers,
            },
            Err(unreachable) => {
                let (state, error) = self.offline_state();
                DaemonOverview {
                    mode: self.mode,
                    state,
                    error,
                    pid: None,
                    status: None,
                    metrics: MetricsReport::default(),
                    prometheus: String::new(),
                    ipc: IpcReport {
                        endpoint,
                        reachable: false,
                        round_trip_us: None,
                        error: Some(unreachable.to_string()),
                    },
                    calculations: Vec::new(),
                    profilers: Vec::new(),
                }
            }
        }
    }

    pub async fn send_ipc(&self, request: IpcRequest) -> Result<IpcExchange, DaemonControlError> {
        let line = serde_json::to_string(&request).map_err(DaemonControlError::Json)?;
        let started = Instant::now();
        let response = self.request(request).await?;
        Ok(IpcExchange {
            request: line,
            response: serde_json::to_string(&response).map_err(DaemonControlError::Json)?,
            round_trip_us: micros(started.elapsed()),
        })
    }

    pub async fn start_profiler(&self, kind: ProfilerKind) -> Result<(), DaemonControlError> {
        self.profiler(service::START_PROFILER, kind, IPC_TIMEOUT)
            .await
    }

    pub async fn stop_profiler(&self, kind: ProfilerKind) -> Result<(), DaemonControlError> {
        self.profiler(service::STOP_PROFILER, kind, PROFILE_TIMEOUT)
            .await
    }

    fn start_embedded(&self) -> Result<(), DaemonControlError> {
        let mut run = lock(&self.embedded);
        if run.as_ref().is_some_and(|run| lock(&run.result).is_none()) {
            return Err(DaemonControlError::AlreadyRunning);
        }
        // Ctrl+C belongs to the app, not to the daemon inside it.
        let daemon = service::build(self.endpoint.clone(), &self.data_dir, SignalConfig::none())?;
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

    async fn launch_detached(&self) -> Result<(), DaemonControlError> {
        if IpcClient::connect(&self.endpoint).await.is_ok() {
            return Ok(());
        }
        let program = self.install_daemon_program()?;
        DetachedDaemon::new(self.endpoint.clone(), program)
            .arg(DAEMON_FLAG)
            .arg(&self.data_dir)
            .log_file(self.daemon_dir().join("daemon.log"))
            .ensure_running()
            .await?;
        Ok(())
    }

    /// Copies this executable for the detached daemon to run. The original stays unlocked, so
    /// a rebuild or an update can replace it while the daemon runs.
    fn install_daemon_program(&self) -> Result<PathBuf, DaemonControlError> {
        let current = env::current_exe().map_err(DaemonControlError::CurrentExe)?;
        let dir = self.daemon_dir();
        fs::create_dir_all(&dir).map_err(|source| DaemonControlError::CreateFolder {
            path: dir.clone(),
            source,
        })?;
        let target = dir.join(current.file_name().unwrap_or(OsStr::new(NAME)));
        match fs::copy(&current, &target) {
            Ok(_) => Ok(target),
            // A copy that is still shutting down keeps its file locked. Start from it as is.
            Err(_) if target.is_file() => Ok(target),
            Err(source) => Err(DaemonControlError::CopyDaemon {
                from: current,
                to: target,
                source,
            }),
        }
    }

    fn daemon_dir(&self) -> PathBuf {
        self.data_dir.join("daemon")
    }

    fn offline_state(&self) -> (DaemonState, Option<String>) {
        match self.mode {
            DaemonMode::Embedded => match lock(&self.embedded).as_ref() {
                None => (DaemonState::Stopped, None),
                Some(run) => match lock(&run.result).as_ref() {
                    Some(Ok(())) => (DaemonState::Stopped, None),
                    Some(Err(error)) => (DaemonState::Failed, Some(error.to_string())),
                    None if run.handle.stats().shutdown_reason.is_some() => {
                        (DaemonState::Stopping, None)
                    }
                    None => (DaemonState::Starting, None),
                },
            },
            DaemonMode::Detached => match self.launch.try_lock() {
                Err(_) => (DaemonState::Starting, None),
                Ok(last_error) => match last_error.as_ref() {
                    Some(error) => (DaemonState::Failed, Some(error.clone())),
                    None => (DaemonState::Stopped, None),
                },
            },
        }
    }

    async fn profiler(
        &self,
        method: &str,
        kind: ProfilerKind,
        timeout: Duration,
    ) -> Result<(), DaemonControlError> {
        let params =
            serde_json::to_value(ProfilerParams { kind }).map_err(DaemonControlError::Json)?;
        self.call::<Value>(method, params, timeout).await?;
        Ok(())
    }

    async fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<T, DaemonControlError> {
        let call = async {
            let mut client = IpcClient::connect(&self.endpoint).await?;
            client.call(method, params).await
        };
        let value = time::timeout(timeout, call)
            .await
            .map_err(|_| DaemonControlError::IpcTimeout(timeout))??;
        serde_json::from_value(value).map_err(DaemonControlError::Json)
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

    use super::{DaemonController, DaemonMode, DaemonOverview, DaemonState, ProfilerKind};
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

    fn embedded(test: &str) -> DaemonController {
        let name = format!("nakama-desktop-{test}-{}", process::id());
        DaemonController::new(
            DaemonMode::Embedded,
            IpcEndpoint::named(&name),
            std::env::temp_dir().join(name),
        )
    }

    #[test]
    fn runs_answers_ipc_and_starts_again() {
        let controller = embedded("run");
        async_runtime::block_on(async {
            controller.start().await.expect("start");
            assert!(matches!(
                controller.start().await,
                Err(DaemonControlError::AlreadyRunning)
            ));

            let overview = wait_for(&controller, |overview| {
                overview.ipc.reachable && !overview.calculations.is_empty()
            })
            .await;
            assert_eq!(overview.state, DaemonState::Running);
            assert_eq!(overview.pid, Some(process::id()));
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
            assert_eq!(overview.profilers.len(), 2);

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

            controller.start().await.expect("start again");
        });
        controller.stop_blocking();
    }

    #[test]
    fn controls_profilers_over_ipc() {
        let controller = embedded("profilers");
        async_runtime::block_on(async {
            controller.start().await.expect("start");
            wait_for(&controller, |overview| overview.ipc.reachable).await;

            #[cfg(not(all(feature = "profiling", unix)))]
            assert!(matches!(
                controller.start_profiler(ProfilerKind::Cpu).await,
                Err(DaemonControlError::Daemon(nakama_daemon::DaemonError::IpcMethod { message, .. }))
                    if message == "the CPU profiler is not available in this build"
            ));

            #[cfg(feature = "heap-profiling")]
            {
                controller
                    .start_profiler(ProfilerKind::Heap)
                    .await
                    .expect("start heap profiler");
                controller
                    .stop_profiler(ProfilerKind::Heap)
                    .await
                    .expect("stop heap profiler");
                let overview = controller.overview().await;
                let profile = overview.profilers[1]
                    .last_profile
                    .as_deref()
                    .expect("profile path");
                assert!(std::path::Path::new(profile).is_file());
            }
        });
        controller.stop_blocking();
    }
}
