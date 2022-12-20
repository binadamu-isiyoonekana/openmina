use serde::{Deserialize, Serialize};

use shared::requests::PendingRequests;

use crate::rpc::{P2pRpcIdType, P2pRpcOutgoingError, P2pRpcRequest, P2pRpcResponse};

type P2pRpcOutgoingContainer = PendingRequests<P2pRpcIdType, P2pRpcOutgoingStatus>;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct P2pRpcOutgoingState(P2pRpcOutgoingContainer);

impl std::ops::Deref for P2pRpcOutgoingState {
    type Target = P2pRpcOutgoingContainer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for P2pRpcOutgoingState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcOutgoingStatus {
    Init {
        time: redux::Timestamp,
        request: P2pRpcRequest,
    },
    Pending {
        time: redux::Timestamp,
        request: P2pRpcRequest,
    },
    Received {
        time: redux::Timestamp,
        request: P2pRpcRequest,
        response: P2pRpcResponse,
    },
    Error {
        time: redux::Timestamp,
        request: P2pRpcRequest,
        response: Option<P2pRpcResponse>,
        error: P2pRpcOutgoingError,
    },
    Success {
        time: redux::Timestamp,
        request: P2pRpcRequest,
        response: P2pRpcResponse,
    },
}

impl P2pRpcOutgoingStatus {
    pub fn is_init(&self) -> bool {
        matches!(self, Self::Init { .. })
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    pub fn is_received(&self) -> bool {
        matches!(self, Self::Received { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Error { .. } | Self::Success { .. })
    }

    pub fn request(&self) -> &P2pRpcRequest {
        match self {
            Self::Init { request, .. } => request,
            Self::Pending { request, .. } => request,
            Self::Received { request, .. } => request,
            Self::Error { request, .. } => request,
            Self::Success { request, .. } => request,
        }
    }
}
