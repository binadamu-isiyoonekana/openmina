use redux::ActionMeta;

use crate::disconnection::{P2pDisconnectionInitAction, P2pDisconnectionReason};
use crate::peer::P2pPeerReadyAction;
use crate::{connection::P2pConnectionService, webrtc};

use super::{
    P2pConnectionIncomingAnswerReadyAction, P2pConnectionIncomingAnswerSdpCreateErrorAction,
    P2pConnectionIncomingAnswerSdpCreatePendingAction,
    P2pConnectionIncomingAnswerSdpCreateSuccessAction,
    P2pConnectionIncomingAnswerSendSuccessAction, P2pConnectionIncomingError,
    P2pConnectionIncomingErrorAction, P2pConnectionIncomingFinalizeErrorAction,
    P2pConnectionIncomingFinalizePendingAction, P2pConnectionIncomingFinalizeSuccessAction,
    P2pConnectionIncomingInitAction, P2pConnectionIncomingLibp2pReceivedAction,
    P2pConnectionIncomingSuccessAction, P2pConnectionIncomingTimeoutAction,
};

impl P2pConnectionIncomingInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingAnswerSdpCreatePendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.opts.peer_id;
        store.service().incoming_init(peer_id, self.opts.offer);
        store.dispatch(P2pConnectionIncomingAnswerSdpCreatePendingAction { peer_id });
    }
}

impl P2pConnectionIncomingAnswerSdpCreateErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionIncomingErrorAction {
            peer_id: self.peer_id,
            error: P2pConnectionIncomingError::SdpCreateError(self.error),
        });
    }
}

impl P2pConnectionIncomingAnswerSdpCreateSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingAnswerReadyAction: redux::EnablingCondition<S>,
    {
        let answer = webrtc::Answer {
            sdp: self.sdp,
            identity_pub_key: store.state().config.identity_pub_key.clone(),
            target_peer_id: self.peer_id,
        };
        store.dispatch(P2pConnectionIncomingAnswerReadyAction {
            peer_id: self.peer_id,
            answer,
        });
    }
}

impl P2pConnectionIncomingAnswerReadyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
    {
        store.service().set_answer(self.peer_id, self.answer);
    }
}

impl P2pConnectionIncomingAnswerSendSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingFinalizePendingAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionIncomingFinalizePendingAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionIncomingFinalizeErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionIncomingErrorAction {
            peer_id: self.peer_id,
            error: P2pConnectionIncomingError::FinalizeError(self.error),
        });
    }
}

impl P2pConnectionIncomingFinalizeSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingSuccessAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionIncomingSuccessAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionIncomingTimeoutAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionIncomingErrorAction {
            peer_id: self.peer_id,
            error: P2pConnectionIncomingError::Timeout,
        });
    }
}

impl P2pConnectionIncomingSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pPeerReadyAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store.dispatch(P2pPeerReadyAction {
            peer_id,
            incoming: true,
        });
    }
}

impl P2pConnectionIncomingLibp2pReceivedAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pPeerReadyAction: redux::EnablingCondition<S>,
        P2pDisconnectionInitAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        if let Err(err) = store.state().libp2p_incoming_accept(peer_id) {
            store.dispatch(P2pDisconnectionInitAction {
                peer_id,
                reason: P2pDisconnectionReason::Libp2pIncomingRejected(err),
            });
        } else {
            store.dispatch(P2pPeerReadyAction {
                peer_id,
                incoming: true,
            });
        }
    }
}
