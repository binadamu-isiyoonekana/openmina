use serde::{Deserialize, Serialize};

use super::{ExternalSnarkWorkerError, ExternalSnarkWorkerWorkError, SnarkWorkId, SnarkWorkResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalSnarkWorkerState {
    None,
    Starting,

    Idle,
    Working(SnarkWorkId),
    WorkReady(SnarkWorkId, SnarkWorkResult),
    WorkError(SnarkWorkId, ExternalSnarkWorkerWorkError),

    Cancelling(SnarkWorkId),
    Cancelled(SnarkWorkId),

    Killing,
    Error(ExternalSnarkWorkerError),
}

impl ExternalSnarkWorkerState {
    pub fn new() -> Self {
        ExternalSnarkWorkerState::None
    }

    pub fn is_idle(&self) -> bool {
        match self {
            ExternalSnarkWorkerState::Idle => true,
            _ => false,
        }
    }

    pub fn has_idle(&self) -> bool {
        self.available() > 0
    }

    pub fn available(&self) -> usize {
        if matches!(self, ExternalSnarkWorkerState::Idle) {
            1
        } else {
            0
        }
    }

    pub fn working_job_id(&self) -> Option<&SnarkWorkId> {
        match self {
            Self::Working(job_id) => Some(job_id),
            _ => None,
        }
    }
}
