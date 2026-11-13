use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::time;
use tracing::{error, info};

use crate::metrics::{self, Metrics};
use crate::shutdown::ShutdownHandle;

/// A long-running unit of work managed by the daemon.
///
/// `run` should return soon after `ShutdownHandle::cancelled` resolves.
pub trait Subsystem: Send + Sync + 'static {
    type Error: Error + Send + Sync + 'static;

    fn name(&self) -> &str;

    fn run(&self, shutdown: ShutdownHandle)
    -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn is_healthy(&self) -> bool {
        true
    }

    fn restart_policy(&self) -> RestartPolicy {
        RestartPolicy::Never
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RestartPolicy {
    #[default]
    Never,
    Always,
    OnFailure,
    /// Restarts after every exit, doubling the delay each time, up to `max_attempts` restarts.
    ExponentialBackoff {
        initial_delay: Duration,
        max_delay: Duration,
        max_attempts: u32,
    },
}

impl RestartPolicy {
    /// The delay before restart number `attempt + 1`, or `None` to stay stopped.
    fn delay(self, attempt: u32, failed: bool) -> Option<Duration> {
        match self {
            Self::Never => None,
            Self::Always => Some(Duration::ZERO),
            Self::OnFailure => failed.then_some(Duration::ZERO),
            Self::ExponentialBackoff {
                initial_delay,
                max_delay,
                max_attempts,
            } => (attempt < max_attempts).then(|| {
                initial_delay
                    .saturating_mul(1 << attempt.min(10))
                    .min(max_delay)
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubsystemState {
    Starting,
    Running,
    Restarting,
    Stopping,
    Stopped,
    Failed,
    /// Still running when the graceful timeout ran out.
    Aborted,
}

impl SubsystemState {
    fn is_active(self) -> bool {
        matches!(
            self,
            Self::Starting | Self::Running | Self::Restarting | Self::Stopping
        )
    }
}

#[derive(Debug, Clone)]
pub struct SubsystemStatus {
    pub name: String,
    pub state: SubsystemState,
    pub restarts: u32,
    pub last_error: Option<Arc<dyn Error + Send + Sync>>,
}

type BoxError = Box<dyn Error + Send + Sync>;
type BoxFuture<'a> = Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'a>>;

/// Object-safe form of `Subsystem`, so subsystems with different error types share one list.
trait DynSubsystem: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, shutdown: ShutdownHandle) -> BoxFuture<'_>;
    fn is_healthy(&self) -> bool;
    fn restart_policy(&self) -> RestartPolicy;
}

impl<S: Subsystem> DynSubsystem for S {
    fn name(&self) -> &str {
        Subsystem::name(self)
    }

    fn run(&self, shutdown: ShutdownHandle) -> BoxFuture<'_> {
        Box::pin(async move { Subsystem::run(self, shutdown).await.map_err(Into::into) })
    }

    fn is_healthy(&self) -> bool {
        Subsystem::is_healthy(self)
    }

    fn restart_policy(&self) -> RestartPolicy {
        Subsystem::restart_policy(self)
    }
}

pub(crate) struct Task<F, E> {
    name: String,
    run: F,
    error: PhantomData<fn() -> E>,
}

impl<F, E> Task<F, E> {
    pub(crate) fn new(name: String, run: F) -> Self {
        Self {
            name,
            run,
            error: PhantomData,
        }
    }
}

impl<F, Fut, E> Subsystem for Task<F, E>
where
    F: Fn(ShutdownHandle) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), E>> + Send,
    E: Error + Send + Sync + 'static,
{
    type Error = E;

    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, shutdown: ShutdownHandle) -> impl Future<Output = Result<(), E>> + Send {
        (self.run)(shutdown)
    }
}

pub(crate) struct Entry {
    subsystem: Box<dyn DynSubsystem>,
    status: Mutex<SubsystemStatus>,
}

impl Entry {
    pub(crate) fn new<S: Subsystem>(subsystem: S) -> Self {
        let status = SubsystemStatus {
            name: Subsystem::name(&subsystem).to_owned(),
            state: SubsystemState::Starting,
            restarts: 0,
            last_error: None,
        };
        Self {
            subsystem: Box::new(subsystem),
            status: Mutex::new(status),
        }
    }

    pub(crate) fn name(&self) -> &str {
        self.subsystem.name()
    }

    pub(crate) fn status(&self) -> SubsystemStatus {
        self.lock().clone()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.lock().state.is_active()
    }

    /// Subsystems that are not running count as healthy.
    pub(crate) fn is_healthy(&self) -> bool {
        self.lock().state != SubsystemState::Running || self.subsystem.is_healthy()
    }

    pub(crate) fn mark_stopping(&self) {
        let mut status = self.lock();
        if status.state == SubsystemState::Running {
            status.state = SubsystemState::Stopping;
        }
    }

    pub(crate) fn mark_aborted(&self) {
        let mut status = self.lock();
        if status.state.is_active() {
            status.state = SubsystemState::Aborted;
        }
    }

    fn set_state(&self, state: SubsystemState) {
        self.lock().state = state;
    }

    fn lock(&self) -> MutexGuard<'_, SubsystemStatus> {
        self.status.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Runs a subsystem until shutdown, restarting it as its policy allows.
pub(crate) async fn supervise(entry: Arc<Entry>, shutdown: ShutdownHandle, metrics: Metrics) {
    let name = entry.name();
    let policy = entry.subsystem.restart_policy();
    let mut attempt = 0;
    loop {
        entry.set_state(SubsystemState::Running);
        info!(subsystem = name, "subsystem started");

        let result = entry.subsystem.run(shutdown.clone()).await;
        let failed = result.is_err();
        let ended = match result {
            Ok(()) => {
                info!(subsystem = name, "subsystem stopped");
                entry.set_state(SubsystemState::Stopped);
                SubsystemState::Stopped
            }
            Err(source) => {
                error!(subsystem = name, error = %source, "subsystem failed");
                metrics.increment_counter(metrics::FAILURES, 1);
                let mut status = entry.lock();
                status.state = SubsystemState::Failed;
                status.last_error = Some(Arc::from(source));
                SubsystemState::Failed
            }
        };

        if shutdown.is_shutdown() {
            return;
        }
        let Some(delay) = policy.delay(attempt, failed) else {
            return;
        };
        attempt += 1;
        metrics.increment_counter(metrics::RESTARTS, 1);
        {
            let mut status = entry.lock();
            status.state = SubsystemState::Restarting;
            status.restarts = attempt;
        }
        info!(subsystem = name, attempt, ?delay, "restarting subsystem");

        tokio::select! {
            () = shutdown.cancelled() => {
                entry.set_state(ended);
                return;
            }
            () = time::sleep(delay) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RestartPolicy;
    use std::time::Duration;

    #[test]
    fn never_and_on_failure() {
        assert_eq!(RestartPolicy::Never.delay(0, true), None);
        assert_eq!(RestartPolicy::OnFailure.delay(0, false), None);
        assert_eq!(
            RestartPolicy::OnFailure.delay(5, true),
            Some(Duration::ZERO)
        );
        assert_eq!(RestartPolicy::Always.delay(5, false), Some(Duration::ZERO));
    }

    #[test]
    fn backoff_doubles_up_to_the_cap() {
        let policy = RestartPolicy::ExponentialBackoff {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(500),
            max_attempts: 4,
        };
        let delays: Vec<_> = (0..5).map(|attempt| policy.delay(attempt, true)).collect();
        assert_eq!(
            delays,
            [
                Some(Duration::from_millis(100)),
                Some(Duration::from_millis(200)),
                Some(Duration::from_millis(400)),
                Some(Duration::from_millis(500)),
                None,
            ]
        );
    }
}
