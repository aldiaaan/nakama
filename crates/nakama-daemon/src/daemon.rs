use std::error::Error;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use tokio::task::JoinSet;
use tokio::time::{self, Instant, Interval, MissedTickBehavior};
use tracing::{error, info, warn};

use crate::config::{Config, ShutdownTimeouts};
use crate::error::DaemonError;
use crate::ipc::IpcServer;
use crate::metrics::{self, Metrics, MetricsSnapshot};
use crate::shutdown::{ShutdownHandle, ShutdownReason};
use crate::signal::{SignalConfig, SignalListener};
use crate::subsystem::{self, Entry, Subsystem, SubsystemState, SubsystemStatus, Task};

pub struct Daemon {
    config: Config,
    signals: SignalConfig,
    shared: Arc<Shared>,
}

pub struct DaemonBuilder {
    config: Config,
    signals: SignalConfig,
    metrics: Metrics,
    subsystems: Vec<Entry>,
}

/// Controls a daemon from outside `Daemon::run`, before, during, or after it.
#[derive(Clone)]
pub struct DaemonHandle {
    shared: Arc<Shared>,
}

#[derive(Debug, Clone)]
pub struct DaemonStats {
    pub name: String,
    pub uptime: Option<Duration>,
    pub shutdown_reason: Option<ShutdownReason>,
    pub subsystems: Vec<SubsystemStatus>,
}

struct Shared {
    name: String,
    shutdown: ShutdownHandle,
    subsystems: Vec<Arc<Entry>>,
    metrics: Metrics,
    started_at: OnceLock<Instant>,
}

impl Daemon {
    pub fn builder(config: Config) -> DaemonBuilder {
        DaemonBuilder {
            config,
            signals: SignalConfig::default(),
            metrics: Metrics::new(),
            subsystems: Vec::new(),
        }
    }

    pub fn handle(&self) -> DaemonHandle {
        DaemonHandle {
            shared: Arc::clone(&self.shared),
        }
    }

    /// Runs every subsystem until a signal or `DaemonHandle::shutdown`, then stops them.
    pub async fn run(self) -> Result<(), DaemonError> {
        let Self {
            config,
            signals,
            shared,
        } = self;

        let mut signals = SignalListener::listen(signals)?;
        let _pid_file = config
            .pid_file
            .as_deref()
            .map(PidFile::create)
            .transpose()?;
        let ipc = config.ipc.as_ref().map(IpcServer::bind).transpose()?;
        let _ = shared.started_at.set(Instant::now());
        info!(
            daemon = %shared.name,
            subsystems = shared.subsystems.len(),
            "daemon started"
        );

        let mut tasks = JoinSet::new();
        for entry in &shared.subsystems {
            tasks.spawn(subsystem::supervise(
                Arc::clone(entry),
                shared.shutdown.clone(),
                shared.metrics.clone(),
            ));
        }
        if let Some(ipc) = ipc {
            let handle = DaemonHandle {
                shared: Arc::clone(&shared),
            };
            tasks.spawn(ipc.serve(handle, shared.shutdown.clone()));
        }

        let mut health = config.health_check_interval.map(|period| {
            let mut interval = time::interval_at(Instant::now() + period, period);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
            interval
        });

        loop {
            tokio::select! {
                () = shared.shutdown.cancelled() => break,
                signal = signals.recv() => {
                    shared.shutdown.initiate(ShutdownReason::Signal(signal));
                }
                () = tick(&mut health) => shared.check_health(),
            }
        }
        drop(signals);

        let result = shared.stop(config.shutdown, tasks).await;
        info!(
            daemon = %shared.name,
            uptime = ?shared.uptime(),
            "daemon stopped"
        );
        result
    }
}

impl DaemonBuilder {
    pub fn with_signals(mut self, signals: SignalConfig) -> Self {
        self.signals = signals;
        self
    }

    pub fn without_signals(self) -> Self {
        self.with_signals(SignalConfig::none())
    }

    /// Shares `metrics` with the daemon, so subsystems that hold a clone report through it.
    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }

    pub fn with_subsystem<S: Subsystem>(mut self, subsystem: S) -> Self {
        self.subsystems.push(Entry::new(subsystem));
        self
    }

    /// Adds a closure as a subsystem that is never restarted.
    pub fn with_task<F, Fut, E>(self, name: impl Into<String>, task: F) -> Self
    where
        F: Fn(ShutdownHandle) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), E>> + Send,
        E: Error + Send + Sync + 'static,
    {
        self.with_subsystem(Task::new(name.into(), task))
    }

    pub fn build(self) -> Result<Daemon, DaemonError> {
        self.config.validate()?;
        self.metrics.increment_counter(metrics::RESTARTS, 0);
        self.metrics.increment_counter(metrics::FAILURES, 0);
        let shared = Arc::new(Shared {
            name: self.config.name.clone(),
            shutdown: ShutdownHandle::new(),
            subsystems: self.subsystems.into_iter().map(Arc::new).collect(),
            metrics: self.metrics,
            started_at: OnceLock::new(),
        });
        Ok(Daemon {
            config: self.config,
            signals: self.signals,
            shared,
        })
    }
}

impl DaemonHandle {
    /// Returns `false` when shutdown had already started.
    pub fn shutdown(&self) -> bool {
        self.shared.shutdown.initiate(ShutdownReason::Requested)
    }

    pub fn is_running(&self) -> bool {
        self.shared.started_at.get().is_some() && !self.shared.shutdown.is_shutdown()
    }

    pub fn stats(&self) -> DaemonStats {
        DaemonStats {
            name: self.shared.name.clone(),
            uptime: self.shared.uptime(),
            shutdown_reason: self.shared.shutdown.reason(),
            subsystems: self
                .shared
                .subsystems
                .iter()
                .map(|entry| entry.status())
                .collect(),
        }
    }

    /// Everything recorded so far, plus the daemon's own uptime and subsystem gauges.
    pub fn metrics(&self) -> MetricsSnapshot {
        let mut snapshot = self.shared.metrics.snapshot();
        let statuses: Vec<_> = self
            .shared
            .subsystems
            .iter()
            .map(|entry| entry.status())
            .collect();
        let count = |state| {
            statuses
                .iter()
                .filter(|status| status.state == state)
                .count() as f64
        };
        let gauges = [
            (
                metrics::UPTIME,
                self.shared
                    .uptime()
                    .map_or(0.0, |uptime| uptime.as_secs_f64()),
            ),
            (metrics::SUBSYSTEMS, statuses.len() as f64),
            (metrics::SUBSYSTEMS_RUNNING, count(SubsystemState::Running)),
            (metrics::SUBSYSTEMS_FAILED, count(SubsystemState::Failed)),
        ];
        for (name, value) in gauges {
            snapshot.gauges.insert(name.to_owned(), value);
        }
        snapshot
    }
}

impl Shared {
    fn uptime(&self) -> Option<Duration> {
        self.started_at.get().map(Instant::elapsed)
    }

    fn check_health(&self) {
        for entry in &self.subsystems {
            if !entry.is_healthy() {
                warn!(subsystem = entry.name(), "subsystem is unhealthy");
            }
        }
    }

    fn active(&self) -> Vec<String> {
        self.subsystems
            .iter()
            .filter(|entry| entry.is_active())
            .map(|entry| entry.name().to_owned())
            .collect()
    }

    async fn stop(
        &self,
        timeouts: ShutdownTimeouts,
        mut tasks: JoinSet<()>,
    ) -> Result<(), DaemonError> {
        let started = self.shutdown.initiated_at().unwrap_or_else(Instant::now);
        for entry in &self.subsystems {
            entry.mark_stopping();
        }

        if drain(&mut tasks, started + timeouts.graceful).await {
            return Ok(());
        }
        warn!(
            subsystems = ?self.active(),
            timeout = ?timeouts.graceful,
            "graceful shutdown timed out, aborting subsystems"
        );
        tasks.abort_all();
        let drained = drain(&mut tasks, started + timeouts.force).await;
        let stuck = self.active();
        for entry in &self.subsystems {
            entry.mark_aborted();
        }
        if drained {
            return Ok(());
        }
        Err(DaemonError::ShutdownTimeout {
            timeout: timeouts.force,
            subsystems: stuck,
        })
    }
}

/// Returns `true` when every task finished before `deadline`.
async fn drain(tasks: &mut JoinSet<()>, deadline: Instant) -> bool {
    let joined = async {
        while let Some(joined) = tasks.join_next().await {
            if let Err(source) = joined
                && source.is_panic()
            {
                error!(error = %source, "subsystem panicked");
            }
        }
    };
    time::timeout_at(deadline, joined).await.is_ok()
}

async fn tick(interval: &mut Option<Interval>) {
    match interval {
        Some(interval) => {
            interval.tick().await;
        }
        None => std::future::pending().await,
    }
}

struct PidFile(PathBuf);

impl PidFile {
    fn create(path: &Path) -> Result<Self, DaemonError> {
        fs::write(path, format!("{}\n", process::id())).map_err(|source| {
            DaemonError::WritePidFile {
                path: path.to_owned(),
                source,
            }
        })?;
        Ok(Self(path.to_owned()))
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::Daemon;
    use crate::config::{Config, ShutdownTimeouts};
    use crate::ipc::{IpcClient, IpcEndpoint, IpcRequest, IpcResponse};
    use crate::metrics::Metrics;
    use crate::shutdown::{ShutdownHandle, ShutdownReason};
    use crate::signal::{Signal, SignalConfig};
    use crate::subsystem::{RestartPolicy, Subsystem, SubsystemState};
    use std::convert::Infallible;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;
    use std::{fs, process};
    use tokio::time;

    #[derive(Debug, thiserror::Error)]
    #[error("boom")]
    struct Boom;

    struct Flaky;

    impl Subsystem for Flaky {
        type Error = Boom;

        fn name(&self) -> &str {
            "flaky"
        }

        async fn run(&self, _shutdown: ShutdownHandle) -> Result<(), Boom> {
            Err(Boom)
        }

        fn restart_policy(&self) -> RestartPolicy {
            RestartPolicy::ExponentialBackoff {
                initial_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(40),
                max_attempts: 3,
            }
        }
    }

    fn config() -> Config {
        Config {
            shutdown: ShutdownTimeouts {
                graceful: Duration::from_millis(100),
                force: Duration::from_millis(200),
            },
            health_check_interval: None,
            ..Config::default()
        }
    }

    #[tokio::test(start_paused = true)]
    async fn stops_tasks_when_shutdown_is_requested() {
        let daemon = Daemon::builder(config())
            .without_signals()
            .with_task("worker", |shutdown: ShutdownHandle| async move {
                shutdown.cancelled().await;
                Ok::<_, Infallible>(())
            })
            .build()
            .expect("build");
        let handle = daemon.handle();
        assert!(!handle.is_running());

        let run = tokio::spawn(daemon.run());
        time::sleep(Duration::from_millis(10)).await;
        assert!(handle.is_running());
        assert_eq!(handle.stats().subsystems[0].state, SubsystemState::Running);

        assert!(handle.shutdown());
        assert!(!handle.shutdown());
        run.await.expect("join").expect("run");

        let stats = handle.stats();
        assert_eq!(stats.shutdown_reason, Some(ShutdownReason::Requested));
        assert_eq!(stats.subsystems[0].state, SubsystemState::Stopped);
        assert!(!handle.is_running());
    }

    #[tokio::test(start_paused = true)]
    async fn restarts_with_backoff_until_attempts_run_out() {
        let daemon = Daemon::builder(config())
            .without_signals()
            .with_subsystem(Flaky)
            .build()
            .expect("build");
        let handle = daemon.handle();
        let run = tokio::spawn(daemon.run());

        time::sleep(Duration::from_secs(1)).await;
        handle.shutdown();
        run.await.expect("join").expect("run");

        let status = &handle.stats().subsystems[0];
        assert_eq!(status.state, SubsystemState::Failed);
        assert_eq!(status.restarts, 3);
        assert_eq!(
            status.last_error.as_ref().map(ToString::to_string),
            Some("boom".to_owned())
        );

        let metrics = handle.metrics();
        assert_eq!(metrics.counters["daemon_subsystem_restarts_total"], 3);
        assert_eq!(metrics.counters["daemon_subsystem_failures_total"], 4);
        assert_eq!(metrics.gauges["daemon_subsystems"], 1.0);
        assert_eq!(metrics.gauges["daemon_subsystems_failed"], 1.0);
    }

    #[tokio::test]
    async fn answers_ipc_requests() {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "nakama-daemon-test-{}-{}",
            process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let endpoint = IpcEndpoint::named(&name);
        let metrics = Metrics::new();
        let daemon = Daemon::builder(Config {
            ipc: Some(endpoint.clone()),
            ..config()
        })
        .without_signals()
        .with_metrics(metrics.clone())
        .with_task("worker", |shutdown: ShutdownHandle| async move {
            shutdown.cancelled().await;
            Ok::<_, Infallible>(())
        })
        .build()
        .expect("build");
        let handle = daemon.handle();
        let run = tokio::spawn(daemon.run());

        let mut client = connect(&endpoint).await;
        let IpcResponse::Status(status) = client.request(IpcRequest::Status).await.expect("status")
        else {
            panic!("expected a status response");
        };
        assert_eq!(status.name, "nakama-daemon");
        assert_eq!(status.subsystems[0].state, SubsystemState::Running);

        metrics.increment_counter("jobs_total", 2);
        let IpcResponse::Metrics { prometheus } =
            client.request(IpcRequest::Metrics).await.expect("metrics")
        else {
            panic!("expected a metrics response");
        };
        assert!(prometheus.contains("jobs_total 2\n"));
        assert!(prometheus.contains("# TYPE daemon_uptime_seconds gauge\n"));

        assert_eq!(
            client
                .request(IpcRequest::Shutdown)
                .await
                .expect("shutdown"),
            IpcResponse::Shutdown { initiated: true }
        );
        run.await.expect("join").expect("run");
        assert_eq!(
            handle.stats().shutdown_reason,
            Some(ShutdownReason::Requested)
        );
        assert!(IpcClient::connect(&endpoint).await.is_err());
    }

    async fn connect(endpoint: &IpcEndpoint) -> IpcClient {
        for _ in 0..100 {
            if let Ok(client) = IpcClient::connect(endpoint).await {
                return client;
            }
            time::sleep(Duration::from_millis(10)).await;
        }
        panic!("daemon never started listening on {endpoint}");
    }

    #[tokio::test(start_paused = true)]
    async fn aborts_subsystems_that_ignore_shutdown() {
        let daemon = Daemon::builder(config())
            .without_signals()
            .with_task("stuck", |_shutdown| async {
                std::future::pending::<()>().await;
                Ok::<_, Infallible>(())
            })
            .build()
            .expect("build");
        let handle = daemon.handle();
        let run = tokio::spawn(daemon.run());

        time::sleep(Duration::from_millis(10)).await;
        handle.shutdown();
        run.await.expect("join").expect("run");

        assert_eq!(handle.stats().subsystems[0].state, SubsystemState::Aborted);
    }

    #[tokio::test(start_paused = true)]
    async fn listens_for_every_signal() {
        let daemon = Daemon::builder(config())
            .with_signals(SignalConfig::default().with(Signal::Hangup))
            .build()
            .expect("build");
        let handle = daemon.handle();
        let run = tokio::spawn(daemon.run());

        time::sleep(Duration::from_millis(10)).await;
        assert!(handle.is_running());
        handle.shutdown();
        run.await.expect("join").expect("run");
    }

    #[tokio::test(start_paused = true)]
    async fn writes_and_removes_the_pid_file() {
        let path = std::env::temp_dir().join(format!("nakama-daemon-{}.pid", process::id()));
        let daemon = Daemon::builder(Config {
            pid_file: Some(path.clone()),
            ..config()
        })
        .without_signals()
        .build()
        .expect("build");
        let handle = daemon.handle();
        let run = tokio::spawn(daemon.run());

        time::sleep(Duration::from_millis(10)).await;
        assert_eq!(
            fs::read_to_string(&path).expect("pid file"),
            format!("{}\n", process::id())
        );

        handle.shutdown();
        run.await.expect("join").expect("run");
        assert!(!path.exists());
    }
}
