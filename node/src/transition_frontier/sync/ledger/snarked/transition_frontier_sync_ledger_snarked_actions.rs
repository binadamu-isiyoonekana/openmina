use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};
use serde::{Deserialize, Serialize};

use crate::ledger::{LedgerAddress, LEDGER_DEPTH};
use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;
use crate::transition_frontier::sync::ledger::TransitionFrontierSyncLedgerState;

use super::{
    PeerLedgerQueryError, PeerLedgerQueryResponse, PeerRpcState,
    TransitionFrontierSyncLedgerSnarkedState,
};

pub type TransitionFrontierSyncLedgerSnarkedActionWithMeta =
    redux::ActionWithMeta<TransitionFrontierSyncLedgerSnarkedAction>;
pub type TransitionFrontierSyncLedgerSnarkedActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierSyncLedgerSnarkedAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerSnarkedAction {
    Pending(TransitionFrontierSyncLedgerSnarkedPendingAction),
    PeersQuery(TransitionFrontierSyncLedgerSnarkedPeersQueryAction),
    PeerQueryInit(TransitionFrontierSyncLedgerSnarkedPeerQueryInitAction),
    PeerQueryPending(TransitionFrontierSyncLedgerSnarkedPeerQueryPendingAction),
    PeerQueryRetry(TransitionFrontierSyncLedgerSnarkedPeerQueryRetryAction),
    PeerQueryError(TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction),
    PeerQuerySuccess(TransitionFrontierSyncLedgerSnarkedPeerQuerySuccessAction),
    ChildHashesReceived(TransitionFrontierSyncLedgerSnarkedChildHashesReceivedAction),
    ChildAccountsReceived(TransitionFrontierSyncLedgerSnarkedChildAccountsReceivedAction),
    Success(TransitionFrontierSyncLedgerSnarkedSuccessAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedPendingAction {}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncLedgerSnarkedPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.transition_frontier.sync.ledger().map_or(false, |s| {
            matches!(s, TransitionFrontierSyncLedgerState::Init { .. })
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedPeersQueryAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedPeersQueryAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        let peers_available = state
            .p2p
            .ready_peers_iter()
            .any(|(_, p)| p.channels.rpc.can_send_request());
        peers_available
            && state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked())
                .map_or(false, |s| {
                    s.sync_next().is_some() || s.sync_retry_iter().next().is_some()
                })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedPeerQueryInitAction {
    pub address: LedgerAddress,
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedPeerQueryInitAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        None.or_else(|| {
            let target_best_tip = state.transition_frontier.sync.best_tip()?;
            let ledger = state.transition_frontier.sync.ledger()?.snarked()?;
            let target = ledger.target();

            // This is true if there is a next address that needs to be queried
            // from a peer and it matches the one requested by this action.
            let check_next_addr = match ledger {
                TransitionFrontierSyncLedgerSnarkedState::Pending {
                    pending, next_addr, ..
                } => next_addr.as_ref().map_or(false, |next_addr| {
                    next_addr == &self.address
                        && (next_addr.to_index().0 != 0 || pending.is_empty())
                }),
                _ => false,
            };

            let peer = state.p2p.get_ready_peer(&self.peer_id)?;
            let check_peer_available = {
                let peer_best_tip = peer.best_tip.as_ref()?;
                if !peer.channels.rpc.can_send_request() {
                    false
                } else if target.staged.is_some() {
                    // if peer has same best tip, then he has same root
                    // so we can sync root snarked+staged ledger from that peer.
                    target_best_tip.hash() == peer_best_tip.hash()
                } else {
                    &target.snarked_ledger_hash == peer_best_tip.snarked_ledger_hash()
                        || &target.snarked_ledger_hash == peer_best_tip.staking_epoch_ledger_hash()
                        || &target.snarked_ledger_hash == peer_best_tip.next_epoch_ledger_hash()
                }
            };

            Some(check_next_addr && check_peer_available)
        })
        .unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedPeerQueryRetryAction {
    pub address: LedgerAddress,
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedPeerQueryRetryAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        None.or_else(|| {
            let target_best_tip = state.transition_frontier.sync.best_tip()?;
            let ledger = state.transition_frontier.sync.ledger()?.snarked()?;
            let target = ledger.target();

            // This is true if there is next retry address and it
            // matches the one requested in this action.
            let check_next_addr = state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked()?.sync_retry_iter().next())
                .map_or(false, |addr| addr == self.address);

            let peer = state.p2p.get_ready_peer(&self.peer_id)?;
            let check_peer_available = {
                let peer_best_tip = peer.best_tip.as_ref()?;
                if !peer.channels.rpc.can_send_request() {
                    false
                } else if target.staged.is_some() {
                    // if peer has same best tip, then he has same root
                    // so we can sync root snarked+staged ledger from that peer.
                    target_best_tip.hash() == peer_best_tip.hash()
                } else {
                    &target.snarked_ledger_hash == peer_best_tip.snarked_ledger_hash()
                        || &target.snarked_ledger_hash == peer_best_tip.staking_epoch_ledger_hash()
                        || &target.snarked_ledger_hash == peer_best_tip.next_epoch_ledger_hash()
                }
            };

            Some(check_next_addr && check_peer_available)
        })
        .unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedPeerQueryPendingAction {
    pub address: LedgerAddress,
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedPeerQueryPendingAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.snarked()?.fetch_pending())
            .map_or(false, |pending| {
                pending
                    .iter()
                    .filter_map(|(_, query_state)| query_state.attempts.get(&self.peer_id))
                    .any(|peer_rpc_state| matches!(peer_rpc_state, PeerRpcState::Init { .. }))
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction {
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
    pub error: PeerLedgerQueryError,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.snarked())
            .map_or(false, |s| {
                s.peer_query_get(&self.peer_id, self.rpc_id)
                    .and_then(|(_, s)| s.attempts.get(&self.peer_id))
                    .map_or(false, |s| matches!(s, PeerRpcState::Pending { .. }))
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedPeerQuerySuccessAction {
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
    pub response: PeerLedgerQueryResponse,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedPeerQuerySuccessAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.snarked())
            .map_or(false, |s| {
                // TODO(binier): check if expected response
                // kind is correct.
                s.peer_query_get(&self.peer_id, self.rpc_id)
                    .and_then(|(_, s)| s.attempts.get(&self.peer_id))
                    .map_or(false, |s| matches!(s, PeerRpcState::Pending { .. }))
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedChildHashesReceivedAction {
    pub address: LedgerAddress,
    pub hashes: (LedgerHash, LedgerHash),
    pub sender: PeerId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedChildHashesReceivedAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.address.length() < LEDGER_DEPTH - 1
            && state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked()?.fetch_pending()?.get(&self.address))
                .and_then(|s| s.attempts.get(&self.sender))
                .map_or(false, |s| s.is_success())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedChildAccountsReceivedAction {
    pub address: LedgerAddress,
    pub accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    pub sender: PeerId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedChildAccountsReceivedAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.snarked()?.fetch_pending()?.get(&self.address))
            .and_then(|s| s.attempts.get(&self.sender))
            // TODO(binier): check if expected response
            // kind is correct.
            .map_or(false, |s| s.is_success())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedSuccessAction {}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncLedgerSnarkedSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.snarked())
            .map_or(false, |s| match s {
                TransitionFrontierSyncLedgerSnarkedState::Pending {
                    pending, next_addr, ..
                } => next_addr.is_none() && pending.is_empty(),
                _ => false,
            })
    }
}

use crate::transition_frontier::{
    sync::{ledger::TransitionFrontierSyncLedgerAction, TransitionFrontierSyncAction},
    TransitionFrontierAction,
};

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::TransitionFrontier(TransitionFrontierAction::Sync(
                    TransitionFrontierSyncAction::Ledger(
                        TransitionFrontierSyncLedgerAction::Snarked(value.into()),
                    ),
                ))
            }
        }
    };
}

impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedPendingAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedPeersQueryAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedPeerQueryInitAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedPeerQueryPendingAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedPeerQueryRetryAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedPeerQuerySuccessAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedChildHashesReceivedAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedChildAccountsReceivedAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedSuccessAction);
