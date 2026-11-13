use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[cfg(all(feature = "profiling", unix))]
use nakama_daemon::CpuProfiler;
#[cfg(feature = "heap-profiling")]
use nakama_daemon::HeapProfiler;
use nakama_daemon::{DaemonError, HeapUsage};
use serde::{Deserialize, Serialize};

use crate::error::DaemonControlError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProfilerKind {
    Cpu,
    Heap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Availability {
    Available,
    /// The CPU profiler samples with signals, which only Linux and macOS have.
    UnsupportedPlatform,
    /// The app was built without the cargo feature that enables this profiler.
    FeatureDisabled,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilerReport {
    pub kind: ProfilerKind,
    pub availability: Availability,
    pub running: bool,
    pub elapsed_ms: Option<u64>,
    pub last_profile: Option<String>,
    pub heap: Option<HeapUsage>,
}

/// Writes the profile once the profiler is out of the lock.
pub type FinishProfile = Box<dyn FnOnce() -> Result<PathBuf, DaemonError> + Send>;

#[derive(Default)]
pub struct Profilers {
    #[cfg(all(feature = "profiling", unix))]
    cpu: Option<Running<CpuProfiler>>,
    #[cfg(feature = "heap-profiling")]
    heap: Option<Running<HeapProfiler>>,
    last_cpu: Option<PathBuf>,
    last_heap: Option<PathBuf>,
}

#[cfg_attr(
    not(any(all(feature = "profiling", unix), feature = "heap-profiling")),
    allow(dead_code)
)]
struct Running<P> {
    profiler: P,
    path: PathBuf,
    started: Instant,
}

impl ProfilerKind {
    fn availability(self) -> Availability {
        match self {
            Self::Cpu if cfg!(all(feature = "profiling", unix)) => Availability::Available,
            Self::Cpu if cfg!(unix) => Availability::FeatureDisabled,
            Self::Cpu => Availability::UnsupportedPlatform,
            Self::Heap if cfg!(feature = "heap-profiling") => Availability::Available,
            Self::Heap => Availability::FeatureDisabled,
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Cpu => "pb",
            Self::Heap => "json",
        }
    }
}

impl fmt::Display for ProfilerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cpu => "CPU",
            Self::Heap => "heap",
        })
    }
}

impl Profilers {
    pub fn start(&mut self, kind: ProfilerKind, dir: &Path) -> Result<(), DaemonControlError> {
        if kind.availability() != Availability::Available {
            return Err(DaemonControlError::ProfilerUnavailable(kind));
        }
        if self.is_running(kind) {
            return Err(DaemonControlError::ProfilerRunning(kind));
        }
        let path = profile_path(dir, kind)?;
        match kind {
            ProfilerKind::Cpu => self.start_cpu(path),
            ProfilerKind::Heap => self.start_heap(path),
        }
    }

    pub fn stop(&mut self, kind: ProfilerKind) -> Result<FinishProfile, DaemonControlError> {
        if kind.availability() != Availability::Available {
            return Err(DaemonControlError::ProfilerUnavailable(kind));
        }
        match kind {
            ProfilerKind::Cpu => self.stop_cpu(),
            ProfilerKind::Heap => self.stop_heap(),
        }
        .ok_or(DaemonControlError::ProfilerIdle(kind))
    }

    pub fn finished(&mut self, kind: ProfilerKind, path: PathBuf) {
        match kind {
            ProfilerKind::Cpu => self.last_cpu = Some(path),
            ProfilerKind::Heap => self.last_heap = Some(path),
        }
    }

    pub fn reports(&self) -> Vec<ProfilerReport> {
        vec![
            self.report(ProfilerKind::Cpu),
            self.report(ProfilerKind::Heap),
        ]
    }

    fn report(&self, kind: ProfilerKind) -> ProfilerReport {
        let last = match kind {
            ProfilerKind::Cpu => &self.last_cpu,
            ProfilerKind::Heap => &self.last_heap,
        };
        ProfilerReport {
            kind,
            availability: kind.availability(),
            running: self.is_running(kind),
            elapsed_ms: self
                .started(kind)
                .map(|started| u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)),
            last_profile: last.as_ref().map(|path| path.display().to_string()),
            heap: match kind {
                ProfilerKind::Cpu => None,
                ProfilerKind::Heap => self.heap_usage(),
            },
        }
    }

    fn is_running(&self, kind: ProfilerKind) -> bool {
        self.started(kind).is_some()
    }

    fn started(&self, kind: ProfilerKind) -> Option<Instant> {
        match kind {
            #[cfg(all(feature = "profiling", unix))]
            ProfilerKind::Cpu => self.cpu.as_ref().map(|running| running.started),
            #[cfg(feature = "heap-profiling")]
            ProfilerKind::Heap => self.heap.as_ref().map(|running| running.started),
            #[allow(unreachable_patterns)]
            _ => None,
        }
    }

    #[cfg(all(feature = "profiling", unix))]
    fn start_cpu(&mut self, path: PathBuf) -> Result<(), DaemonControlError> {
        self.cpu = Some(Running {
            profiler: CpuProfiler::start()?,
            path,
            started: Instant::now(),
        });
        Ok(())
    }

    #[cfg(not(all(feature = "profiling", unix)))]
    fn start_cpu(&mut self, _path: PathBuf) -> Result<(), DaemonControlError> {
        Err(DaemonControlError::ProfilerUnavailable(ProfilerKind::Cpu))
    }

    #[cfg(all(feature = "profiling", unix))]
    fn stop_cpu(&mut self) -> Option<FinishProfile> {
        let Running { profiler, path, .. } = self.cpu.take()?;
        Some(Box::new(move || {
            profiler.stop(&path)?;
            Ok(path)
        }))
    }

    #[cfg(not(all(feature = "profiling", unix)))]
    fn stop_cpu(&mut self) -> Option<FinishProfile> {
        None
    }

    #[cfg(feature = "heap-profiling")]
    fn start_heap(&mut self, path: PathBuf) -> Result<(), DaemonControlError> {
        self.heap = Some(Running {
            profiler: HeapProfiler::start(&path)?,
            path,
            started: Instant::now(),
        });
        Ok(())
    }

    #[cfg(not(feature = "heap-profiling"))]
    fn start_heap(&mut self, _path: PathBuf) -> Result<(), DaemonControlError> {
        Err(DaemonControlError::ProfilerUnavailable(ProfilerKind::Heap))
    }

    #[cfg(feature = "heap-profiling")]
    fn stop_heap(&mut self) -> Option<FinishProfile> {
        let Running { profiler, path, .. } = self.heap.take()?;
        // dhat writes the profile when the profiler drops.
        Some(Box::new(move || {
            drop(profiler);
            Ok(path)
        }))
    }

    #[cfg(not(feature = "heap-profiling"))]
    fn stop_heap(&mut self) -> Option<FinishProfile> {
        None
    }

    #[cfg(feature = "heap-profiling")]
    fn heap_usage(&self) -> Option<HeapUsage> {
        self.heap.as_ref().map(|running| running.profiler.usage())
    }

    #[cfg(not(feature = "heap-profiling"))]
    fn heap_usage(&self) -> Option<HeapUsage> {
        None
    }
}

fn profile_path(dir: &Path, kind: ProfilerKind) -> Result<PathBuf, DaemonControlError> {
    fs::create_dir_all(dir).map_err(|source| DaemonControlError::ProfilesFolder {
        path: dir.to_owned(),
        source,
    })?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let name = match kind {
        ProfilerKind::Cpu => "cpu",
        ProfilerKind::Heap => "heap",
    };
    Ok(dir.join(format!("{name}-{stamp}.{}", kind.extension())))
}
