use serde::Serialize;

#[cfg(all(feature = "profiling", unix))]
pub use cpu::CpuProfiler;
#[cfg(feature = "heap-profiling")]
pub use heap::{HeapProfiler, HeapProfilerAlloc};

/// Allocations seen since a heap profiler started. All zero without the global allocator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeapUsage {
    pub total_bytes: u64,
    pub total_blocks: u64,
    pub current_bytes: u64,
    pub current_blocks: u64,
    pub max_bytes: u64,
    pub max_blocks: u64,
}

#[cfg(all(feature = "profiling", unix))]
mod cpu {
    use std::fs;
    use std::path::Path;

    use pprof::ProfilerGuard;
    use pprof::protos::Message;

    use crate::error::DaemonError;

    const SAMPLES_PER_SECOND: i32 = 99;

    /// Samples every thread's stack until stopped. Only one can run per process.
    pub struct CpuProfiler {
        guard: ProfilerGuard<'static>,
    }

    impl CpuProfiler {
        pub fn start() -> Result<Self, DaemonError> {
            let guard = ProfilerGuard::new(SAMPLES_PER_SECOND).map_err(DaemonError::CpuProfiler)?;
            Ok(Self { guard })
        }

        /// Writes a protobuf profile that `go tool pprof` and similar viewers open.
        pub fn stop(self, path: impl AsRef<Path>) -> Result<(), DaemonError> {
            let path = path.as_ref();
            let report = self
                .guard
                .report()
                .build()
                .map_err(DaemonError::CpuProfiler)?;
            let profile = report.pprof().map_err(DaemonError::CpuProfiler)?;
            fs::write(path, profile.encode_to_vec()).map_err(|source| DaemonError::WriteProfile {
                path: path.to_owned(),
                source,
            })
        }
    }
}

#[cfg(feature = "heap-profiling")]
mod heap {
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::HeapUsage;
    use crate::error::DaemonError;

    pub use dhat::Alloc as HeapProfilerAlloc;

    static RUNNING: AtomicBool = AtomicBool::new(false);

    /// Tracks allocations until dropped, then writes a DHAT JSON profile.
    ///
    /// The binary must declare
    /// `#[global_allocator] static ALLOC: HeapProfilerAlloc = HeapProfilerAlloc;`,
    /// otherwise the profile is empty.
    pub struct HeapProfiler {
        profiler: Option<dhat::Profiler>,
    }

    impl HeapProfiler {
        pub fn start(path: impl AsRef<Path>) -> Result<Self, DaemonError> {
            if RUNNING.swap(true, Ordering::AcqRel) {
                return Err(DaemonError::HeapProfilerRunning);
            }
            let profiler = dhat::Profiler::builder().file_name(path).build();
            Ok(Self {
                profiler: Some(profiler),
            })
        }

        pub fn usage(&self) -> HeapUsage {
            let stats = dhat::HeapStats::get();
            HeapUsage {
                total_bytes: stats.total_bytes,
                total_blocks: stats.total_blocks,
                current_bytes: stats.curr_bytes as u64,
                current_blocks: stats.curr_blocks as u64,
                max_bytes: stats.max_bytes as u64,
                max_blocks: stats.max_blocks as u64,
            }
        }
    }

    impl Drop for HeapProfiler {
        fn drop(&mut self) {
            drop(self.profiler.take());
            RUNNING.store(false, Ordering::Release);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::HeapProfiler;
        use crate::error::DaemonError;
        use std::{fs, process};

        #[test]
        fn allows_one_profiler_at_a_time() {
            let path =
                std::env::temp_dir().join(format!("nakama-daemon-heap-{}.json", process::id()));
            let profiler = HeapProfiler::start(&path).expect("start");
            let _ = profiler.usage();
            assert!(matches!(
                HeapProfiler::start(&path),
                Err(DaemonError::HeapProfilerRunning)
            ));
            drop(profiler);
            assert!(path.is_file());
            drop(HeapProfiler::start(&path).expect("restart"));
            let _ = fs::remove_file(path);
        }
    }
}
