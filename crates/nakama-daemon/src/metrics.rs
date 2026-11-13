use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::{Arc, PoisonError, RwLock, RwLockWriteGuard};
use std::time::Duration;

use tokio::time::Instant;

pub(crate) const UPTIME: &str = "daemon_uptime_seconds";
pub(crate) const SUBSYSTEMS: &str = "daemon_subsystems";
pub(crate) const SUBSYSTEMS_RUNNING: &str = "daemon_subsystems_running";
pub(crate) const SUBSYSTEMS_FAILED: &str = "daemon_subsystems_failed";
pub(crate) const RESTARTS: &str = "daemon_subsystem_restarts_total";
pub(crate) const FAILURES: &str = "daemon_subsystem_failures_total";

/// Counters, gauges, and timings shared by the daemon and its subsystems.
///
/// Characters that Prometheus does not allow in a metric name become `_`.
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    registry: Arc<RwLock<MetricsSnapshot>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MetricsSnapshot {
    pub counters: BTreeMap<String, u64>,
    pub gauges: BTreeMap<String, f64>,
    pub timings: BTreeMap<String, Timing>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Timing {
    pub count: u64,
    pub total: Duration,
}

/// Records the time since it was started when dropped.
#[derive(Debug)]
pub struct Timer {
    metrics: Metrics,
    name: String,
    started: Instant,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_counter(&self, name: &str, by: u64) {
        let mut registry = self.write();
        let counter = registry.counters.entry(metric_name(name)).or_default();
        *counter = counter.saturating_add(by);
    }

    pub fn set_gauge(&self, name: &str, value: f64) {
        self.write().gauges.insert(metric_name(name), value);
    }

    pub fn record_duration(&self, name: &str, duration: Duration) {
        let mut registry = self.write();
        let timing = registry.timings.entry(metric_name(name)).or_default();
        timing.count = timing.count.saturating_add(1);
        timing.total = timing.total.saturating_add(duration);
    }

    pub fn start_timer(&self, name: &str) -> Timer {
        Timer {
            metrics: self.clone(),
            name: name.to_owned(),
            started: Instant::now(),
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        self.registry
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn write(&self) -> RwLockWriteGuard<'_, MetricsSnapshot> {
        self.registry
            .write()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.metrics
            .record_duration(&self.name, self.started.elapsed());
    }
}

impl MetricsSnapshot {
    /// Prometheus text exposition format, version 0.0.4. Timings render as summaries in seconds.
    pub fn render_prometheus(&self) -> String {
        let mut out = String::new();
        for (name, value) in &self.counters {
            let _ = writeln!(out, "# TYPE {name} counter\n{name} {value}");
        }
        for (name, value) in &self.gauges {
            let _ = writeln!(out, "# TYPE {name} gauge\n{name} {value}");
        }
        for (name, timing) in &self.timings {
            let _ = writeln!(
                out,
                "# TYPE {name} summary\n{name}_count {}\n{name}_sum {}",
                timing.count,
                timing.total.as_secs_f64()
            );
        }
        out
    }
}

fn metric_name(name: &str) -> String {
    let mut sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == ':' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if !sanitized.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == ':') {
        sanitized.insert(0, '_');
    }
    sanitized
}

#[cfg(test)]
mod tests {
    use super::{Metrics, Timing, metric_name};
    use std::time::Duration;

    #[test]
    fn records_counters_gauges_and_timings() {
        let metrics = Metrics::new();
        metrics.increment_counter("jobs_total", 5);
        metrics.increment_counter("jobs_total", 3);
        metrics.set_gauge("queue_depth", 2.0);
        metrics.set_gauge("queue_depth", 4.5);
        metrics.record_duration("job", Duration::from_millis(100));
        metrics.record_duration("job", Duration::from_millis(200));

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.counters["jobs_total"], 8);
        assert_eq!(snapshot.gauges["queue_depth"], 4.5);
        assert_eq!(
            snapshot.timings["job"],
            Timing {
                count: 2,
                total: Duration::from_millis(300),
            }
        );
    }

    #[test]
    fn timer_records_on_drop() {
        let metrics = Metrics::new();
        drop(metrics.start_timer("sync"));
        assert_eq!(metrics.snapshot().timings["sync"].count, 1);
    }

    #[test]
    fn sanitizes_names() {
        assert_eq!(metric_name("nakama-agent.jobs"), "nakama_agent_jobs");
        assert_eq!(metric_name("9lives"), "_9lives");
        assert_eq!(metric_name(""), "_");
    }

    #[test]
    fn renders_prometheus_text() {
        let metrics = Metrics::new();
        metrics.increment_counter("jobs_total", 2);
        metrics.set_gauge("queue_depth", 1.5);
        metrics.record_duration("job", Duration::from_millis(250));
        assert_eq!(
            metrics.snapshot().render_prometheus(),
            "# TYPE jobs_total counter\njobs_total 2\n\
             # TYPE queue_depth gauge\nqueue_depth 1.5\n\
             # TYPE job summary\njob_count 1\njob_sum 0.25\n"
        );
    }
}
