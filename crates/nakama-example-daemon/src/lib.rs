mod error;
mod example;
mod workload;

pub use error::WorkloadError;
pub use example::{ExampleDaemon, ExampleReport, NAME};
pub use workload::{Calculation, Outcome, Workload};
