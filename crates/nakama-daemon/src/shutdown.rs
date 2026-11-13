use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use tokio::time::Instant;
use tracing::info;

use crate::signal::Signal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShutdownReason {
    Signal(Signal),
    Requested,
}

impl fmt::Display for ShutdownReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signal(signal) => write!(f, "received {signal}"),
            Self::Requested => f.write_str("requested"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Shutdown {
    reason: ShutdownReason,
    at: Instant,
}

/// Lets a subsystem watch for shutdown. Every clone sees the same state.
#[derive(Debug, Clone)]
pub struct ShutdownHandle {
    state: Arc<watch::Sender<Option<Shutdown>>>,
}

impl ShutdownHandle {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(watch::Sender::new(None)),
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.state.borrow().is_some()
    }

    pub fn reason(&self) -> Option<ShutdownReason> {
        self.state.borrow().map(|shutdown| shutdown.reason)
    }

    /// Resolves once shutdown starts, or right away if it already has.
    pub async fn cancelled(&self) {
        let mut receiver = self.state.subscribe();
        let _ = receiver.wait_for(Option::is_some).await;
    }

    pub(crate) fn initiated_at(&self) -> Option<Instant> {
        self.state.borrow().map(|shutdown| shutdown.at)
    }

    /// Returns `false` when shutdown had already started. The first reason is kept.
    pub(crate) fn initiate(&self, reason: ShutdownReason) -> bool {
        let initiated = self.state.send_if_modified(|state| {
            if state.is_some() {
                return false;
            }
            *state = Some(Shutdown {
                reason,
                at: Instant::now(),
            });
            true
        });
        if initiated {
            info!(%reason, "shutdown initiated");
        }
        initiated
    }
}

#[cfg(test)]
mod tests {
    use super::{ShutdownHandle, ShutdownReason};
    use crate::signal::Signal;

    #[test]
    fn keeps_the_first_reason() {
        let shutdown = ShutdownHandle::new();
        assert!(!shutdown.is_shutdown());
        assert!(shutdown.initiate(ShutdownReason::Requested));
        assert!(!shutdown.initiate(ShutdownReason::Signal(Signal::Terminate)));
        assert!(shutdown.is_shutdown());
        assert_eq!(shutdown.reason(), Some(ShutdownReason::Requested));
    }

    #[tokio::test]
    async fn cancelled_resolves_for_every_clone() {
        let shutdown = ShutdownHandle::new();
        let waiter = shutdown.clone();
        let task = tokio::spawn(async move { waiter.cancelled().await });
        shutdown.initiate(ShutdownReason::Requested);
        task.await.expect("join");
        shutdown.cancelled().await;
    }
}
