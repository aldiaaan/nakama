use thiserror::Error;
use tokio::task::JoinError;

use crate::workload::Workload;

/// Why an example calculation subsystem exited. The daemon restarts it.
#[derive(Debug, Error)]
pub enum WorkloadError {
    #[error("rejected random input {seed} for {workload}")]
    RejectedInput { workload: Workload, seed: u32 },

    #[error("{workload} calculation stopped unexpectedly: {source}")]
    Join {
        workload: Workload,
        #[source]
        source: JoinError,
    },
}
