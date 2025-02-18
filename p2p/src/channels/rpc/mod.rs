mod p2p_channels_rpc_state;
pub use p2p_channels_rpc_state::*;

mod p2p_channels_rpc_actions;
pub use p2p_channels_rpc_actions::*;

mod p2p_channels_rpc_reducer;
pub use p2p_channels_rpc_reducer::*;

mod p2p_channels_rpc_effects;
pub use p2p_channels_rpc_effects::*;

use std::{sync::Arc, time::Duration};

use binprot_derive::{BinProtRead, BinProtWrite};
use mina_p2p_messages::v2::{
    LedgerHash, MinaBasePendingCoinbaseStableV2, MinaBaseStateBodyHashStableV1,
    MinaLedgerSyncLedgerAnswerStableV2, MinaLedgerSyncLedgerQueryStableV1,
    MinaStateProtocolStateValueStableV2, StateHash, TransactionSnarkScanStateStableV2,
};
use openmina_core::{
    block::ArcBlock,
    snark::{Snark, SnarkJobId},
};
use serde::{Deserialize, Serialize};

use crate::connection::outgoing::P2pConnectionOutgoingInitOpts;

pub type P2pRpcId = u32;

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum RpcChannelMsg {
    Request(P2pRpcId, P2pRpcRequest),
    Response(P2pRpcId, Option<P2pRpcResponse>),
}

impl RpcChannelMsg {
    pub fn request_id(&self) -> P2pRpcId {
        match self {
            Self::Request(id, _) => *id,
            Self::Response(id, _) => *id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum P2pRpcKind {
    BestTipWithProof,
    LedgerQuery,
    StagedLedgerAuxAndPendingCoinbasesAtBlock,
    Block,
    Snark,
    InitialPeers,
}

impl P2pRpcKind {
    pub fn timeout(self) -> Option<Duration> {
        match self {
            Self::BestTipWithProof => Some(Duration::from_secs(10)),
            Self::LedgerQuery => Some(Duration::from_secs(2)),
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock => Some(Duration::from_secs(120)),
            Self::Block => Some(Duration::from_secs(5)),
            Self::Snark => Some(Duration::from_secs(5)),
            Self::InitialPeers => Some(Duration::from_secs(5)),
        }
    }

    pub fn supported_by_libp2p(self) -> bool {
        match self {
            Self::BestTipWithProof => true,
            Self::LedgerQuery => true,
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock => true,
            Self::Block => true,
            Self::Snark => false,
            Self::InitialPeers => true,
        }
    }
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcRequest {
    BestTipWithProof,
    LedgerQuery(LedgerHash, MinaLedgerSyncLedgerQueryStableV1),
    StagedLedgerAuxAndPendingCoinbasesAtBlock(StateHash),
    Block(StateHash),
    Snark(SnarkJobId),
    InitialPeers,
}

impl P2pRpcRequest {
    pub fn kind(&self) -> P2pRpcKind {
        match self {
            Self::BestTipWithProof => P2pRpcKind::BestTipWithProof,
            Self::LedgerQuery(..) => P2pRpcKind::LedgerQuery,
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock(_) => {
                P2pRpcKind::StagedLedgerAuxAndPendingCoinbasesAtBlock
            }
            Self::Block(_) => P2pRpcKind::Block,
            Self::Snark(_) => P2pRpcKind::Snark,
            Self::InitialPeers => P2pRpcKind::InitialPeers,
        }
    }
}

impl Default for P2pRpcRequest {
    fn default() -> Self {
        Self::BestTipWithProof
    }
}

impl std::fmt::Display for P2pRpcRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind())?;
        match self {
            Self::BestTipWithProof => Ok(()),
            Self::LedgerQuery(ledger_hash, query) => {
                let addr_to_str = |addr| {
                    let addr = ledger::Address::from(addr);
                    format!("depth: {}, addr: {}", addr.length(), addr.to_string())
                };

                match query {
                    MinaLedgerSyncLedgerQueryStableV1::NumAccounts => write!(f, ", NumAccounts, ")?,
                    MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(addr) => {
                        write!(f, ", ChildHashes, {}, ", addr_to_str(addr))?
                    }
                    MinaLedgerSyncLedgerQueryStableV1::WhatContents(addr) => {
                        write!(f, ", ChildContents, {}, ", addr_to_str(addr))?
                    }
                }
                write!(f, "ledger: {ledger_hash}")
            }
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock(block_hash)
            | Self::Block(block_hash) => {
                write!(f, ", {block_hash}")
            }
            Self::Snark(job_id) => {
                write!(f, ", {job_id}")
            }
            Self::InitialPeers => Ok(()),
        }
    }
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct BestTipWithProof {
    pub best_tip: ArcBlock,
    pub proof: (Vec<MinaBaseStateBodyHashStableV1>, ArcBlock),
}

/// Pieces required to reconstruct staged ledger from snarked ledger.
#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct StagedLedgerAuxAndPendingCoinbases {
    pub scan_state: TransactionSnarkScanStateStableV2,
    pub staged_ledger_hash: LedgerHash,
    pub pending_coinbase: MinaBasePendingCoinbaseStableV2,
    pub needed_blocks: Vec<MinaStateProtocolStateValueStableV2>,
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcResponse {
    BestTipWithProof(BestTipWithProof),
    LedgerQuery(MinaLedgerSyncLedgerAnswerStableV2),
    StagedLedgerAuxAndPendingCoinbasesAtBlock(Arc<StagedLedgerAuxAndPendingCoinbases>),
    Block(ArcBlock),
    Snark(Snark),
    InitialPeers(Vec<P2pConnectionOutgoingInitOpts>),
}

impl P2pRpcResponse {
    pub fn kind(&self) -> P2pRpcKind {
        match self {
            Self::BestTipWithProof(_) => P2pRpcKind::BestTipWithProof,
            Self::LedgerQuery(_) => P2pRpcKind::LedgerQuery,
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock(_) => {
                P2pRpcKind::StagedLedgerAuxAndPendingCoinbasesAtBlock
            }
            Self::Block(_) => P2pRpcKind::Block,
            Self::Snark(_) => P2pRpcKind::Snark,
            Self::InitialPeers(_) => P2pRpcKind::InitialPeers,
        }
    }
}
