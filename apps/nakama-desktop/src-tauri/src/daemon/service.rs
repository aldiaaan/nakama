use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use nakama_daemon::{
    Config, Daemon, DaemonError, DaemonHandle, IpcEndpoint, Metrics, MetricsSnapshot,
    ShutdownTimeouts, SignalConfig, StatusReport,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::async_runtime;

use super::lock;
use super::profilers::{ProfilerKind, ProfilerReport, Profilers};
use super::workload::{Calculation, CalculationLog, Calculator, Workload};
use crate::error::DaemonControlError;

pub const NAME: &str = "nakama-daemon";
pub const SNAPSHOT: &str = "snapshot";
pub const START_PROFILER: &str = "startProfiler";
pub const STOP_PROFILER: &str = "stopProfiler";

/// Everything the Daemon page reads from the daemon, whichever process it runs in.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub pid: u32,
    pub status: StatusReport,
    pub metrics: MetricsReport,
    pub prometheus: String,
    pub calculations: Vec<Calculation>,
    pub profilers: Vec<ProfilerReport>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsReport {
    pub counters: BTreeMap<String, u64>,
    pub gauges: BTreeMap<String, f64>,
    pub timings: BTreeMap<String, TimingReport>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimingReport {
    pub count: u64,
    pub total_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfilerParams {
    pub kind: ProfilerKind,
}

/// The example daemon: four calculation subsystems, plus IPC methods for the Daemon page.
pub fn build(
    endpoint: IpcEndpoint,
    data_dir: &Path,
    signals: SignalConfig,
) -> Result<Daemon, DaemonError> {
    let metrics = Metrics::new();
    let calculations = CalculationLog::default();
    let profilers = Arc::new(Mutex::new(Profilers::default()));
    let profiles_dir = data_dir.join("profiles");

    let builder = Workload::ALL.into_iter().fold(
        Daemon::builder(Config {
            name: NAME.to_owned(),
            shutdown: ShutdownTimeouts {
                graceful: Duration::from_secs(2),
                force: Duration::from_secs(4),
            },
            ipc: Some(endpoint),
            ..Config::default()
        })
        .with_signals(signals)
        .with_metrics(metrics.clone()),
        |builder, workload| {
            builder.with_subsystem(Calculator::new(
                workload,
                metrics.clone(),
                calculations.clone(),
            ))
        },
    );

    builder
        .with_ipc_method(SNAPSHOT, {
            let profilers = Arc::clone(&profilers);
            move |daemon: DaemonHandle, _params| {
                let snapshot = snapshot(&daemon, &calculations, &profilers);
                async move { serde_json::to_value(snapshot) }
            }
        })
        .with_ipc_method(START_PROFILER, {
            let profilers = Arc::clone(&profilers);
            move |_daemon, params| {
                let started = start_profiler(&profilers, &profiles_dir, params);
                async move { started.map(|()| Value::Null) }
            }
        })
        .with_ipc_method(STOP_PROFILER, move |_daemon, params| {
            stop_profiler(Arc::clone(&profilers), params)
        })
        .build()
}

/// Runs the daemon in this process until a signal or an IPC shutdown request.
pub fn run_standalone(endpoint: IpcEndpoint, data_dir: PathBuf) -> Result<(), DaemonError> {
    let daemon = build(endpoint, &data_dir, SignalConfig::default())?;
    async_runtime::block_on(daemon.run())
}

fn snapshot(
    daemon: &DaemonHandle,
    calculations: &CalculationLog,
    profilers: &Mutex<Profilers>,
) -> Snapshot {
    let metrics = daemon.metrics();
    Snapshot {
        pid: process::id(),
        status: StatusReport::from(&daemon.stats()),
        prometheus: metrics.render_prometheus(),
        metrics: MetricsReport::from(metrics),
        calculations: calculations.recent(),
        profilers: lock(profilers).reports(),
    }
}

fn start_profiler(
    profilers: &Mutex<Profilers>,
    profiles_dir: &Path,
    params: Value,
) -> Result<(), DaemonControlError> {
    let ProfilerParams { kind } =
        serde_json::from_value(params).map_err(DaemonControlError::Json)?;
    lock(profilers).start(kind, profiles_dir)
}

async fn stop_profiler(
    profilers: Arc<Mutex<Profilers>>,
    params: Value,
) -> Result<Value, DaemonControlError> {
    let ProfilerParams { kind } =
        serde_json::from_value(params).map_err(DaemonControlError::Json)?;
    let finish = lock(&profilers).stop(kind)?;
    let path = async_runtime::spawn_blocking(finish)
        .await
        .map_err(DaemonControlError::Join)??;
    lock(&profilers).finished(kind, path);
    Ok(Value::Null)
}

impl From<MetricsSnapshot> for MetricsReport {
    fn from(snapshot: MetricsSnapshot) -> Self {
        Self {
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
        }
    }
}
