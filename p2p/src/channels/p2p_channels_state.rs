use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    best_tip::P2pChannelsBestTipState, snark_job_commitment::P2pChannelsSnarkJobCommitmentState,
    ChannelId,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsState {
    pub best_tip: P2pChannelsBestTipState,
    pub snark_job_commitment: P2pChannelsSnarkJobCommitmentState,
}

impl P2pChannelsState {
    pub fn new(enabled_channels: &BTreeSet<ChannelId>) -> Self {
        Self {
            best_tip: match enabled_channels.contains(&ChannelId::BestTipPropagation) {
                false => P2pChannelsBestTipState::Disabled,
                true => P2pChannelsBestTipState::Enabled,
            },
            snark_job_commitment: match enabled_channels
                .contains(&ChannelId::SnarkJobCommitmentPropagation)
            {
                false => P2pChannelsSnarkJobCommitmentState::Disabled,
                true => P2pChannelsSnarkJobCommitmentState::Enabled,
            },
        }
    }
}

impl P2pChannelsState {
    pub fn is_channel_ready(&self, chan_id: ChannelId) -> bool {
        match chan_id {
            ChannelId::BestTipPropagation => self.snark_job_commitment.is_ready(),
            ChannelId::SnarkJobCommitmentPropagation => self.snark_job_commitment.is_ready(),
        }
    }
}
