use std::sync::Arc;

use serde::{Deserialize, Serialize};

use shared::{block::BlockHeaderWithHash, requests::PendingRequests};

use crate::{VerifierIndex, VerifierSRS};

use super::{SnarkBlockVerifyError, SnarkBlockVerifyId, SnarkBlockVerifyIdType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkBlockVerifyState {
    pub verifier_index: Arc<VerifierIndex>,
    pub verifier_srs: Arc<VerifierSRS>,
    pub jobs: PendingRequests<SnarkBlockVerifyIdType, SnarkBlockVerifyStatus>,
}

impl SnarkBlockVerifyState {
    pub fn new(verifier_index: Arc<VerifierIndex>, verifier_srs: Arc<VerifierSRS>) -> Self {
        Self {
            verifier_index,
            verifier_srs,
            jobs: Default::default(),
        }
    }

    pub fn next_req_id(&self) -> SnarkBlockVerifyId {
        self.jobs.next_req_id()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkBlockVerifyStatus {
    Init {
        time: redux::Timestamp,
        // TODO(binier): use Rc<_> or Arc<_>,
        block: BlockHeaderWithHash,
    },
    Pending {
        time: redux::Timestamp,
        block: BlockHeaderWithHash,
    },
    Error {
        time: redux::Timestamp,
        block: BlockHeaderWithHash,
        error: SnarkBlockVerifyError,
    },
    Success {
        time: redux::Timestamp,
        block: BlockHeaderWithHash,
    },
}

impl SnarkBlockVerifyStatus {
    pub fn is_init(&self) -> bool {
        matches!(self, Self::Init { .. })
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Error { .. } | Self::Success { .. })
    }

    pub fn block(&self) -> &BlockHeaderWithHash {
        match self {
            Self::Init { block, .. } => block,
            Self::Pending { block, .. } => block,
            Self::Error { block, .. } => block,
            Self::Success { block, .. } => block,
        }
    }
}
