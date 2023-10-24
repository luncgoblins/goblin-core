use std::fmt::{Display, Formatter, Result};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint64, Addr, Uint128, CosmosMsg, StdResult, StdError};
use crate::constants::proposal_constants::*;

use self::helpers::is_safe_link;

/// This structure stores data for a proposal.
#[cw_serde]
pub struct Proposal {
    pub proposal_id: Uint64,
    pub submitter: Addr,
    pub status: ProposalStatus,
    pub yes_power: Uint128,
    pub no_power: Uint128,
    pub yes_voters: Vec<Addr>,
    pub no_voters: Vec<Addr>,
    pub start_block: u64,
    pub start_time: u64,
    pub end_block: u64,
    pub delayed_end_block: u64,
    pub expiration_block: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub messages: Option<Vec<CosmosMsg>>,
    pub deposit_amount: Uint128,
}

impl Proposal {
    pub fn validate(&self) -> StdResult<()> {
        // Title validation
        if self.title.len() < MIN_TITLE_LENGTH {
            return Err(StdError::generic_err("Title too short!"));
        }
        if self.title.len() > MAX_TITLE_LENGTH {
            return Err(StdError::generic_err("Title too long!"));
        }
        if !self.title.chars().all(|c| {
            c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || SAFE_TEXT_CHARS.contains(c)
        }) {
            return Err(StdError::generic_err(
                "Title is not in alphanumeric format!",
            ));
        }

        // Description validation
        if self.description.len() < MIN_DESC_LENGTH {
            return Err(StdError::generic_err("Description too short!"));
        }
        if self.description.len() > MAX_DESC_LENGTH {
            return Err(StdError::generic_err("Description too long!"));
        }
        if !self.description.chars().all(|c| {
            c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || SAFE_TEXT_CHARS.contains(c)
        }) {
            return Err(StdError::generic_err(
                "Description is not in alphanumeric format",
            ));
        }

        // Link validation
        if let Some(link) = &self.link {
            if link.len() < MIN_LINK_LENGTH {
                return Err(StdError::generic_err("Link too short!"));
            }
            if link.len() > MAX_LINK_LENGTH {
                return Err(StdError::generic_err("Link too long!"));
            }
            if !is_safe_link(link) {
                return Err(StdError::generic_err(
                    "Link is not properly formatted or contains unsafe characters!",
                ));
            }
        }

        Ok(())
    }
}

/// This enum describes available statuses/states for a Proposal.
#[cw_serde]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    InProgress,
    Failed,
    Executed,
    Expired,
}

impl Display for ProposalStatus {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        match self {
            ProposalStatus::Active {} => fmt.write_str("active"),
            ProposalStatus::Passed {} => fmt.write_str("passed"),
            ProposalStatus::Rejected {} => fmt.write_str("rejected"),
            ProposalStatus::InProgress => fmt.write_str("in_progress"),
            ProposalStatus::Failed => fmt.write_str("failed"),
            ProposalStatus::Executed {} => fmt.write_str("executed"),
            ProposalStatus::Expired {} => fmt.write_str("expired"),
        }
    }
}

/// This structure describes a proposal vote.
#[cw_serde]
pub struct ProposalVote {
    /// Voted option for the proposal
    pub option: ProposalVoteOption,
    /// Vote power
    pub power: Uint128,
}

/// This enum describes available options for voting on a proposal.
#[cw_serde]
pub enum ProposalVoteOption {
    For,
    Against,
}

impl Display for ProposalVoteOption {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        match self {
            ProposalVoteOption::For {} => fmt.write_str("for"),
            ProposalVoteOption::Against {} => fmt.write_str("against"),
        }
    }
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

pub mod helpers {
    use cosmwasm_std::{StdError, StdResult};

    const SAFE_LINK_CHARS: &str = "-_:/?#@!$&()*+,;=.~[]'%";

    /// Checks if the link is valid. Returns a boolean value.
    pub fn is_safe_link(link: &str) -> bool {
        link.chars()
            .all(|c| c.is_ascii_alphanumeric() || SAFE_LINK_CHARS.contains(c))
    }

    /// Validating the list of links. Returns an error if a list has an invalid link.
    pub fn validate_links(links: &[String]) -> StdResult<()> {
        for link in links {
            if !(is_safe_link(link) && link.contains('.') && link.ends_with('/')) {
                return Err(StdError::generic_err(format!(
                    "Link is not properly formatted or contains unsafe characters: {link}."
                )));
            }
        }

        Ok(())
    }
}