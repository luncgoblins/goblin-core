use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, CosmosMsg};

use crate::{proposal::ProposalVoteOption, config::UpdateConfig};

#[cw_serde]
pub struct InstantiateMsg {
    pub staking_addr: String,
    pub vesting_addr: String,
    pub gov_token_denom: String,
    pub proposal_voting_period: u64,
    pub proposal_effective_delay: u64,
    pub proposal_expiration_period: u64,
    pub proposal_required_deposit: Uint128,
    pub proposal_required_quorum: String,
    pub proposal_required_threshold: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SubmitProposal {
        title: String,
        description: String,
        link: Option<String>,
        messages: Option<Vec<CosmosMsg>>,
    },
    CastVote {
        proposal_id: u64,
        vote: ProposalVoteOption,
    },
    EndProposal {
        proposal_id: u64,
    },
    CheckMessages {
        messages: Vec<CosmosMsg>,
    },
    CheckMessagesPassed {},
    ExecuteProposal {
        proposal_id: u64,
    },
    RemoveCompletedProposal {
        proposal_id: u64,
    },
    UpdateConfig(Box<UpdateConfig>),
}