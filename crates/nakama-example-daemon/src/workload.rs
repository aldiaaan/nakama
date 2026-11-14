use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use nakama_daemon::{Metrics, RestartPolicy, ShutdownHandle, Subsystem};
use serde::Serialize;
use tokio::{task, time};

use crate::error::WorkloadError;

const HISTORY: usize = 24;
const PAUSE: Duration = Duration::from_millis(400);
/// About one seed in this many is rejected, so the UI shows failures and restarts.
const REJECT_ONE_IN: u32 = 40;

pub(crate) const JOBS: &str = "workload_jobs_total";
pub(crate) const REJECTED: &str = "workload_rejected_inputs_total";
pub(crate) const PI_ESTIMATE: &str = "workload_pi_estimate";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Workload {
    Primes,
    MonteCarlo,
    Matrix,
    Collatz,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Outcome {
    Primes {
        limit: u64,
        count: u64,
        largest: u64,
    },
    MonteCarlo {
        samples: u64,
        inside: u64,
        estimate: f64,
    },
    Matrix {
        size: u64,
        trace: f64,
        checksum: f64,
    },
    Collatz {
        start: u64,
        span: u64,
        longest: u64,
        steps: u64,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Calculation {
    pub id: u64,
    pub subsystem: &'static str,
    pub seed: u32,
    pub duration_us: u64,
    pub finished_at_ms: u64,
    pub outcome: Outcome,
}

/// The most recent calculations from every subsystem. Survives daemon restarts.
#[derive(Debug, Clone, Default)]
pub struct CalculationLog {
    inner: Arc<Mutex<LogState>>,
}

#[derive(Debug, Default)]
struct LogState {
    next_id: u64,
    recent: VecDeque<Calculation>,
}

pub struct Calculator {
    workload: Workload,
    metrics: Metrics,
    log: CalculationLog,
}

impl Workload {
    pub const ALL: [Self; 4] = [Self::Primes, Self::MonteCarlo, Self::Matrix, Self::Collatz];

    fn name(self) -> &'static str {
        match self {
            Self::Primes => "primes",
            Self::MonteCarlo => "monte-carlo",
            Self::Matrix => "matrix",
            Self::Collatz => "collatz",
        }
    }

    fn timing(self) -> &'static str {
        match self {
            Self::Primes => "workload_primes",
            Self::MonteCarlo => "workload_monte_carlo",
            Self::Matrix => "workload_matrix",
            Self::Collatz => "workload_collatz",
        }
    }

    /// Every input is derived from `seed`, so a seed reproduces its calculation.
    fn compute(self, seed: u32) -> Outcome {
        let mut rng = fastrand::Rng::with_seed(u64::from(seed));
        match self {
            Self::Primes => {
                let limit = rng.u64(200_000..=2_000_000);
                let (count, largest) = sieve(limit);
                Outcome::Primes {
                    limit,
                    count,
                    largest,
                }
            }
            Self::MonteCarlo => {
                let samples = rng.u64(200_000..=2_000_000);
                let inside = (0..samples)
                    .filter(|_| {
                        let (x, y) = (rng.f64(), rng.f64());
                        x * x + y * y <= 1.0
                    })
                    .count() as u64;
                Outcome::MonteCarlo {
                    samples,
                    inside,
                    estimate: 4.0 * inside as f64 / samples as f64,
                }
            }
            Self::Matrix => {
                let size = rng.usize(60..=160);
                let (trace, checksum) = multiply(&mut rng, size);
                Outcome::Matrix {
                    size: size as u64,
                    trace,
                    checksum,
                }
            }
            Self::Collatz => {
                let start = rng.u64(1_000_000..=1_000_000_000);
                let span = rng.u64(20_000..=100_000);
                let (longest, steps) = (start..start + span)
                    .map(|n| (n, collatz_steps(n)))
                    .max_by_key(|&(_, steps)| steps)
                    .unwrap_or((start, 0));
                Outcome::Collatz {
                    start,
                    span,
                    longest,
                    steps,
                }
            }
        }
    }
}

impl fmt::Display for Workload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl CalculationLog {
    /// Newest first.
    pub fn recent(&self) -> Vec<Calculation> {
        self.lock().recent.iter().cloned().collect()
    }

    fn push(&self, workload: Workload, seed: u32, duration: Duration, outcome: Outcome) {
        let mut state = self.lock();
        state.next_id += 1;
        let calculation = Calculation {
            id: state.next_id,
            subsystem: workload.name(),
            seed,
            duration_us: u64::try_from(duration.as_micros()).unwrap_or(u64::MAX),
            finished_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |since| {
                    u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
                }),
            outcome,
        };
        state.recent.push_front(calculation);
        state.recent.truncate(HISTORY);
    }

    fn lock(&self) -> MutexGuard<'_, LogState> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Calculator {
    pub fn new(workload: Workload, metrics: Metrics, log: CalculationLog) -> Self {
        Self {
            workload,
            metrics,
            log,
        }
    }
}

impl Subsystem for Calculator {
    type Error = WorkloadError;

    fn name(&self) -> &str {
        self.workload.name()
    }

    async fn run(&self, shutdown: ShutdownHandle) -> Result<(), WorkloadError> {
        let workload = self.workload;
        let mut rng = fastrand::Rng::new();
        while !shutdown.is_shutdown() {
            let seed = rng.u32(..);
            if seed.is_multiple_of(REJECT_ONE_IN) {
                self.metrics.increment_counter(REJECTED, 1);
                return Err(WorkloadError::RejectedInput { workload, seed });
            }

            let started = Instant::now();
            let outcome = task::spawn_blocking(move || workload.compute(seed))
                .await
                .map_err(|source| WorkloadError::Join { workload, source })?;
            let duration = started.elapsed();

            self.metrics.record_duration(workload.timing(), duration);
            self.metrics.increment_counter(JOBS, 1);
            if let Outcome::MonteCarlo { estimate, .. } = outcome {
                self.metrics.set_gauge(PI_ESTIMATE, estimate);
            }
            self.log.push(workload, seed, duration, outcome);

            if time::timeout(PAUSE, shutdown.cancelled()).await.is_ok() {
                break;
            }
        }
        Ok(())
    }

    fn restart_policy(&self) -> RestartPolicy {
        RestartPolicy::ExponentialBackoff {
            initial_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(4),
            max_attempts: u32::MAX,
        }
    }
}

/// Counts the primes up to `limit` and returns the count and the largest one.
fn sieve(limit: u64) -> (u64, u64) {
    let limit = limit as usize;
    let mut composite = vec![false; limit + 1];
    let (mut count, mut largest) = (0, 0);
    for n in 2..=limit {
        if composite[n] {
            continue;
        }
        count += 1;
        largest = n;
        let mut multiple = n.saturating_mul(n);
        while multiple <= limit {
            composite[multiple] = true;
            multiple += n;
        }
    }
    (count, largest as u64)
}

/// Multiplies two random `size` x `size` matrices and returns the product's trace and sum.
fn multiply(rng: &mut fastrand::Rng, size: usize) -> (f64, f64) {
    let mut random = || {
        (0..size * size)
            .map(|_| rng.f64() * 2.0 - 1.0)
            .collect::<Vec<_>>()
    };
    let (a, b) = (random(), random());
    let mut product = vec![0.0; size * size];
    for row in 0..size {
        for k in 0..size {
            let left = a[row * size + k];
            for column in 0..size {
                product[row * size + column] += left * b[k * size + column];
            }
        }
    }
    let trace = (0..size).map(|i| product[i * size + i]).sum();
    (trace, product.iter().sum())
}

fn collatz_steps(mut n: u64) -> u64 {
    let mut steps = 0;
    while n != 1 {
        n = if n.is_multiple_of(2) {
            n / 2
        } else {
            3 * n + 1
        };
        steps += 1;
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::{Outcome, Workload, collatz_steps, sieve};

    #[test]
    fn counts_primes() {
        assert_eq!(sieve(10), (4, 7));
        assert_eq!(sieve(100), (25, 97));
    }

    #[test]
    fn counts_collatz_steps() {
        assert_eq!(collatz_steps(1), 0);
        assert_eq!(collatz_steps(27), 111);
    }

    #[test]
    fn a_seed_reproduces_its_calculation() {
        for workload in Workload::ALL {
            assert_eq!(workload.compute(7), workload.compute(7));
        }
    }

    #[test]
    fn estimates_pi() {
        let Outcome::MonteCarlo { estimate, .. } = Workload::MonteCarlo.compute(1) else {
            panic!("expected a Monte Carlo outcome");
        };
        assert!((estimate - std::f64::consts::PI).abs() < 0.05);
    }
}
