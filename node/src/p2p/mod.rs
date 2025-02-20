pub use ::p2p::*;

pub mod channels;
pub mod connection;
pub mod disconnection;
pub mod discovery;
pub mod listen;
pub mod peer;

mod p2p_effects;
pub use p2p_effects::*;

impl<S> redux::SubStore<crate::State, P2pState> for crate::Store<S>
where
    S: redux::Service,
{
    type SubAction = P2pAction;
    type Service = S;

    fn state(&self) -> &P2pState {
        &self.state.get().p2p
    }

    fn service(&mut self) -> &mut Self::Service {
        &mut self.service
    }

    fn state_and_service(&mut self) -> (&P2pState, &mut Self::Service) {
        (&self.state.get().p2p, &mut self.service)
    }

    fn dispatch<A>(&mut self, action: A) -> bool
    where
        A: Into<P2pAction> + redux::EnablingCondition<crate::State>,
    {
        crate::Store::sub_dispatch(self, action)
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::P2p(value.into())
            }
        }
    };
}

impl_into_global_action!(listen::P2pListenNewAction);
impl_into_global_action!(listen::P2pListenExpiredAction);
impl_into_global_action!(listen::P2pListenErrorAction);
impl_into_global_action!(listen::P2pListenClosedAction);

impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingRandomInitAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingInitAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingReconnectAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingOfferSdpCreatePendingAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingOfferSdpCreateErrorAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingOfferSdpCreateSuccessAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingOfferReadyAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingOfferSendSuccessAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingAnswerRecvPendingAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingAnswerRecvErrorAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingAnswerRecvSuccessAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingFinalizePendingAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingFinalizeErrorAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingFinalizeSuccessAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingTimeoutAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingErrorAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingSuccessAction);

impl_into_global_action!(connection::incoming::P2pConnectionIncomingInitAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingAnswerSdpCreatePendingAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingAnswerSdpCreateErrorAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingAnswerSdpCreateSuccessAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingAnswerReadyAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingAnswerSendSuccessAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingFinalizePendingAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingFinalizeErrorAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingFinalizeSuccessAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingTimeoutAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingErrorAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingSuccessAction);
impl_into_global_action!(connection::incoming::P2pConnectionIncomingLibp2pReceivedAction);

impl_into_global_action!(disconnection::P2pDisconnectionInitAction);
impl_into_global_action!(disconnection::P2pDisconnectionFinishAction);

impl_into_global_action!(discovery::P2pDiscoveryInitAction);
impl_into_global_action!(discovery::P2pDiscoverySuccessAction);
impl_into_global_action!(discovery::P2pDiscoveryKademliaBootstrapAction);
impl_into_global_action!(discovery::P2pDiscoveryKademliaInitAction);
impl_into_global_action!(discovery::P2pDiscoveryKademliaSuccessAction);
impl_into_global_action!(discovery::P2pDiscoveryKademliaFailureAction);
impl_into_global_action!(discovery::P2pDiscoveryKademliaAddRouteAction);

impl_into_global_action!(channels::P2pChannelsMessageReceivedAction);

impl_into_global_action!(channels::best_tip::P2pChannelsBestTipInitAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipPendingAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipReadyAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipRequestSendAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipReceivedAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipRequestReceivedAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipResponseSendAction);

impl_into_global_action!(channels::snark::P2pChannelsSnarkInitAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkPendingAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkReadyAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkRequestSendAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkPromiseReceivedAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkReceivedAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkRequestReceivedAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkResponseSendAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkLibp2pReceivedAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkLibp2pBroadcastAction);

impl_into_global_action!(channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentInitAction);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentPendingAction
);
impl_into_global_action!(channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentReadyAction);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentRequestSendAction
);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentPromiseReceivedAction
);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentReceivedAction
);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentRequestReceivedAction
);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentResponseSendAction
);

impl_into_global_action!(channels::rpc::P2pChannelsRpcInitAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcPendingAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcReadyAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcRequestSendAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcTimeoutAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcResponseReceivedAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcRequestReceivedAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcResponseSendAction);
