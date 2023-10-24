#![allow(unused_imports)]
use cosmwasm_schema::{QueryResponses, cw_serde};
use cosmwasm_std::{Uint64, Uint128, Addr};
use crate::{config::Config, proposal::{ProposalVoteOption, Proposal}};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(ProposalListResponse)]
    Proposals {
        start: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Vec<Addr>)]
    ProposalVoters {
        proposal_id: u64,
        vote_option: ProposalVoteOption,
        start: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Proposal)]
    Proposal { proposal_id: u64 },
    #[returns(ProposalVotesResponse)]
    ProposalVotes { proposal_id: u64 },
    #[returns(Uint128)]
    UserVotingPower { user: String, proposal_id: u64 },
    #[returns(Uint128)]
    TotalVotingPower { proposal_id: u64 },
}

/// This structure describes a proposal vote response.
#[cw_serde]
pub struct ProposalVotesResponse {
    /// Proposal identifier
    pub proposal_id: u64,
    /// Total amount of `for` votes for a proposal
    pub for_power: Uint128,
    /// Total amount of `against` votes for a proposal.
    pub against_power: Uint128,
}

/// This structure describes a proposal list response.
#[cw_serde]
pub struct ProposalListResponse {
    /// The amount of proposals returned
    pub proposal_count: Uint64,
    /// The list of proposals that are returned
    pub proposal_list: Vec<Proposal>,
}