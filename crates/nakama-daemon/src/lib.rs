// nakama
// Copyright (C) 2026  aldiaaan <62921668+aldiaaan@users.noreply.github.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod config;
mod daemon;
mod detach;
mod error;
mod ipc;
mod lock;
mod metrics;
mod profiling;
mod shutdown;
mod signal;
mod subsystem;

pub use config::{Config, ShutdownTimeouts};
pub use daemon::{Daemon, DaemonBuilder, DaemonHandle, DaemonStats};
pub use detach::{DetachedDaemon, Launch};
pub use error::DaemonError;
pub use ipc::{IpcClient, IpcEndpoint, IpcRequest, IpcResponse, StatusReport, SubsystemReport};
pub use lock::InstanceLock;
pub use metrics::{Metrics, MetricsSnapshot, Timer, Timing};
#[cfg(all(feature = "profiling", unix))]
pub use profiling::CpuProfiler;
pub use profiling::HeapUsage;
#[cfg(feature = "heap-profiling")]
pub use profiling::{HeapProfiler, HeapProfilerAlloc};
pub use shutdown::{ShutdownHandle, ShutdownReason};
pub use signal::{Signal, SignalConfig};
pub use subsystem::{RestartPolicy, Subsystem, SubsystemState, SubsystemStatus};
