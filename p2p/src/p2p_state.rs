use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use openmina_core::requests::RpcId;

use crate::channels::rpc::P2pRpcId;
use crate::channels::{ChannelId, P2pChannelsState};
use crate::connection::outgoing::P2pConnectionOutgoingInitOpts;
use crate::PeerId;

use super::connection::P2pConnectionState;
use super::P2pConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pState {
    pub config: P2pConfig,
    pub peers: BTreeMap<PeerId, P2pPeerState>,
    pub known_peers: BTreeMap<PeerId, P2pConnectionOutgoingInitOpts>,
    pub kademlia: P2pKademliaState,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pKademliaState {
    pub is_ready: bool,
    pub last_used: Option<redux::Timestamp>,
    pub outgoing_requests: usize,
}

impl P2pState {
    pub fn enough_time_elapsed(&self, time: redux::Timestamp) -> bool {
        let Some(last_used) = self.kademlia.last_used else {
            return true;
        };
        time.checked_sub(last_used)
            .map(|t| t > self.config.ask_initial_peers_interval)
            .unwrap_or(false)
    }
}

impl P2pState {
    pub fn new(config: P2pConfig) -> Self {
        let known_peers = config
            .initial_peers
            .iter()
            .map(|p| (p.peer_id().clone(), p.clone()))
            .collect();
        Self {
            config,
            peers: Default::default(),
            known_peers,
            kademlia: P2pKademliaState::default(),
        }
    }

    pub fn peer_connection_rpc_id(&self, peer_id: &PeerId) -> Option<RpcId> {
        self.peers.get(peer_id)?.connection_rpc_id()
    }

    /// Get peer in ready state. `None` if peer isn't in `Ready` state,
    /// or if peer doesn't exist.
    pub fn get_ready_peer(&self, peer_id: &PeerId) -> Option<&P2pPeerStatusReady> {
        self.peers.get(peer_id)?.status.as_ready()
    }

    /// Get peer in ready state. `None` if peer isn't in `Ready` state,
    /// or if peer doesn't exist.
    pub fn get_ready_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut P2pPeerStatusReady> {
        self.peers.get_mut(peer_id)?.status.as_ready_mut()
    }

    pub fn any_ready_peers(&self) -> bool {
        self.peers
            .iter()
            .any(|(_, p)| p.status.as_ready().is_some())
    }

    pub fn initial_unused_peers(&self) -> Vec<P2pConnectionOutgoingInitOpts> {
        self.known_peers
            .values()
            .filter(|v| {
                self.ready_peers_iter()
                    .find(|(id, _)| (*id).eq(v.peer_id()))
                    .is_none()
            })
            .cloned()
            .collect()
    }

    pub fn ready_peers_iter(&self) -> impl Iterator<Item = (&PeerId, &P2pPeerStatusReady)> {
        self.peers
            .iter()
            .filter_map(|(id, p)| Some((id, p.status.as_ready()?)))
    }

    pub fn ready_rpc_peers_iter(&self) -> impl '_ + Iterator<Item = (PeerId, P2pRpcId)> {
        self.ready_peers_iter()
            .filter(|(_, p)| p.channels.rpc.can_send_request())
            .map(|(peer_id, p)| (*peer_id, p.channels.rpc.next_local_rpc_id()))
    }

    pub fn ready_peers(&self) -> Vec<PeerId> {
        self.peers
            .iter()
            .filter(|(_, p)| p.status.as_ready().is_some())
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn connected_or_connecting_peers_count(&self) -> usize {
        self.peers
            .iter()
            .filter(|(_, p)| p.status.is_connected_or_connecting())
            .count()
    }

    pub fn is_peer_connected_or_connecting(&self, peer_id: &PeerId) -> bool {
        self.peers
            .get(peer_id)
            .map_or(false, |p| p.status.is_connected_or_connecting())
    }

    pub fn is_libp2p_peer(&self, peer_id: &PeerId) -> bool {
        self.peers.get(peer_id).map_or(false, |p| p.is_libp2p())
    }

    pub fn is_peer_rpc_timed_out(
        &self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
        now: redux::Timestamp,
    ) -> bool {
        self.get_ready_peer(peer_id)
            .map_or(false, |p| p.channels.rpc.is_timed_out(rpc_id, now))
    }

    pub fn peer_rpc_timeouts(&self, now: redux::Timestamp) -> Vec<(PeerId, P2pRpcId)> {
        self.ready_peers_iter()
            .filter_map(|(peer_id, s)| {
                let rpc_id = s.channels.rpc.pending_local_rpc_id()?;
                if !s.channels.rpc.is_timed_out(rpc_id, now) {
                    return None;
                }

                Some((*peer_id, rpc_id))
            })
            .collect()
    }

    pub fn already_has_min_peers(&self) -> bool {
        self.connected_or_connecting_peers_count() >= self.min_peers()
    }

    pub fn already_has_max_peers(&self) -> bool {
        self.connected_or_connecting_peers_count() >= self.config.max_peers
    }

    /// Minimal number of peers that the node should connect
    pub fn min_peers(&self) -> usize {
        (self.config.max_peers / 2).max(3)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerState {
    pub dial_opts: Option<P2pConnectionOutgoingInitOpts>,
    pub status: P2pPeerStatus,
}

impl P2pPeerState {
    pub fn is_libp2p(&self) -> bool {
        self.dial_opts
            .as_ref()
            .map_or(false, |opts| opts.is_libp2p())
    }

    pub fn connection_rpc_id(&self) -> Option<RpcId> {
        match &self.status {
            P2pPeerStatus::Connecting(v) => v.rpc_id(),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pPeerStatus {
    Connecting(P2pConnectionState),
    Disconnected { time: redux::Timestamp },

    Ready(P2pPeerStatusReady),
}

impl P2pPeerStatus {
    /// Checks if the peer is in `Connecting` state and we have finished
    /// connecting to the peer.
    pub fn is_connecting_success(&self) -> bool {
        match self {
            Self::Connecting(v) => v.is_success(),
            _ => false,
        }
    }

    pub fn is_connected_or_connecting(&self) -> bool {
        match self {
            Self::Connecting(s) => !s.is_error(),
            Self::Ready(_) => true,
            Self::Disconnected { .. } => false,
        }
    }

    pub fn as_connecting(&self) -> Option<&P2pConnectionState> {
        match self {
            Self::Connecting(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_ready(&self) -> Option<&P2pPeerStatusReady> {
        match self {
            Self::Ready(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_ready_mut(&mut self) -> Option<&mut P2pPeerStatusReady> {
        match self {
            Self::Ready(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerStatusReady {
    pub connected_since: redux::Timestamp,
    pub last_asked_initial_peers: Option<redux::Timestamp>,
    pub last_received_initial_peers: Option<redux::Timestamp>,
    pub channels: P2pChannelsState,
    pub best_tip: Option<ArcBlockWithHash>,
}

impl P2pPeerStatusReady {
    pub fn new(time: redux::Timestamp, enabled_channels: &BTreeSet<ChannelId>) -> Self {
        Self {
            connected_since: time,
            last_asked_initial_peers: None,
            last_received_initial_peers: None,
            channels: P2pChannelsState::new(enabled_channels),
            best_tip: None,
        }
    }
}
