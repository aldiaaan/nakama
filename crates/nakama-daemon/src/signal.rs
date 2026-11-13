use std::fmt;

use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::error::DaemonError;

/// Process signals that start a graceful shutdown.
///
/// On Windows, `Interrupt` is Ctrl+C, `Quit` is Ctrl+Break, `Terminate` is a
/// console close or system shutdown, and `Hangup` is a user logoff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Signal {
    Interrupt,
    Terminate,
    Quit,
    Hangup,
}

impl Signal {
    const ALL: [Self; 4] = [Self::Interrupt, Self::Terminate, Self::Quit, Self::Hangup];

    pub fn as_str(self) -> &'static str {
        if cfg!(windows) {
            match self {
                Self::Interrupt => "Ctrl+C",
                Self::Terminate => "console close or system shutdown",
                Self::Quit => "Ctrl+Break",
                Self::Hangup => "logoff",
            }
        } else {
            match self {
                Self::Interrupt => "SIGINT",
                Self::Terminate => "SIGTERM",
                Self::Quit => "SIGQUIT",
                Self::Hangup => "SIGHUP",
            }
        }
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Which signals the daemon listens for. Hangup is off by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalConfig {
    interrupt: bool,
    terminate: bool,
    quit: bool,
    hangup: bool,
}

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            interrupt: true,
            terminate: true,
            quit: true,
            hangup: false,
        }
    }
}

impl SignalConfig {
    pub fn none() -> Self {
        Self {
            interrupt: false,
            terminate: false,
            quit: false,
            hangup: false,
        }
    }

    pub fn with(mut self, signal: Signal) -> Self {
        *self.slot(signal) = true;
        self
    }

    pub fn without(mut self, signal: Signal) -> Self {
        *self.slot(signal) = false;
        self
    }

    pub fn is_enabled(mut self, signal: Signal) -> bool {
        *self.slot(signal)
    }

    fn slot(&mut self, signal: Signal) -> &mut bool {
        match signal {
            Signal::Interrupt => &mut self.interrupt,
            Signal::Terminate => &mut self.terminate,
            Signal::Quit => &mut self.quit,
            Signal::Hangup => &mut self.hangup,
        }
    }
}

/// Stops listening when dropped.
pub(crate) struct SignalListener {
    streams: JoinSet<Option<Signal>>,
}

impl SignalListener {
    pub(crate) fn listen(config: SignalConfig) -> Result<Self, DaemonError> {
        let mut streams = JoinSet::new();
        for signal in Signal::ALL {
            if config.is_enabled(signal) {
                spawn(&mut streams, signal)?;
            }
        }
        Ok(Self { streams })
    }

    /// Never resolves when no signal is enabled.
    pub(crate) async fn recv(&mut self) -> Signal {
        while let Some(joined) = self.streams.join_next().await {
            if let Ok(Some(signal)) = joined {
                return signal;
            }
        }
        std::future::pending().await
    }
}

#[cfg(unix)]
fn spawn(streams: &mut JoinSet<Option<Signal>>, signal: Signal) -> Result<(), DaemonError> {
    use tokio::signal::unix::{SignalKind, signal as listen};

    let kind = match signal {
        Signal::Interrupt => SignalKind::interrupt(),
        Signal::Terminate => SignalKind::terminate(),
        Signal::Quit => SignalKind::quit(),
        Signal::Hangup => SignalKind::hangup(),
    };
    let mut stream = listen(kind).map_err(|source| DaemonError::ListenSignal { signal, source })?;
    streams.spawn(async move { stream.recv().await.map(|()| signal) });
    Ok(())
}

#[cfg(windows)]
fn spawn(streams: &mut JoinSet<Option<Signal>>, signal: Signal) -> Result<(), DaemonError> {
    use tokio::signal::windows;

    let error = |source| DaemonError::ListenSignal { signal, source };
    match signal {
        Signal::Interrupt => {
            let mut stream = windows::ctrl_c().map_err(error)?;
            streams.spawn(async move { stream.recv().await.map(|()| signal) });
        }
        Signal::Quit => {
            let mut stream = windows::ctrl_break().map_err(error)?;
            streams.spawn(async move { stream.recv().await.map(|()| signal) });
        }
        Signal::Terminate => {
            let mut close = windows::ctrl_close().map_err(error)?;
            let mut shutdown = windows::ctrl_shutdown().map_err(error)?;
            streams.spawn(async move { close.recv().await.map(|()| signal) });
            streams.spawn(async move { shutdown.recv().await.map(|()| signal) });
        }
        Signal::Hangup => {
            let mut stream = windows::ctrl_logoff().map_err(error)?;
            streams.spawn(async move { stream.recv().await.map(|()| signal) });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Signal, SignalConfig};

    #[test]
    fn default_skips_hangup() {
        let config = SignalConfig::default();
        assert!(config.is_enabled(Signal::Interrupt));
        assert!(config.is_enabled(Signal::Terminate));
        assert!(config.is_enabled(Signal::Quit));
        assert!(!config.is_enabled(Signal::Hangup));
    }

    #[test]
    fn toggles_signals() {
        let config = SignalConfig::default()
            .with(Signal::Hangup)
            .without(Signal::Interrupt);
        assert!(config.is_enabled(Signal::Hangup));
        assert!(!config.is_enabled(Signal::Interrupt));
        assert!(!SignalConfig::none().is_enabled(Signal::Terminate));
    }
}
