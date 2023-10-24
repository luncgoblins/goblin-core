use std::str::FromStr;

use cosmwasm_std::{StdError, Decimal, StdResult, Addr, Uint128};
use crate::constants::proposal_constants::*;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Config {
    pub staking_addr: Addr,
    pub vesting_addr: Addr,
    pub gov_token_denom: String,
    pub proposal_voting_period: u64,
    pub proposal_effective_delay: u64,
    pub proposal_expiration_period: u64,
    pub proposal_required_deposit: Uint128,
    pub proposal_required_quorum: Decimal,
    pub proposal_required_threshold: Decimal,
}

impl Config {
    pub fn validate(&self) -> StdResult<()> {
        if self.proposal_required_threshold
            > Decimal::percent(MAX_PROPOSAL_REQUIRED_THRESHOLD_PERCENTAGE)
            || self.proposal_required_threshold
                < Decimal::percent(MINIMUM_PROPOSAL_REQUIRED_THRESHOLD_PERCENTAGE)
        {
            return Err(StdError::generic_err(format!(
                "The required threshold for a proposal cannot be lower than {MINIMUM_PROPOSAL_REQUIRED_THRESHOLD_PERCENTAGE}% or higher than {MAX_PROPOSAL_REQUIRED_THRESHOLD_PERCENTAGE}%"
            )));
        }

        let max_quorum = Decimal::from_str(MAX_PROPOSAL_REQUIRED_QUORUM_PERCENTAGE)?;
        let min_quorum = Decimal::from_str(MINIMUM_PROPOSAL_REQUIRED_QUORUM_PERCENTAGE)?;
        if self.proposal_required_quorum > max_quorum || self.proposal_required_quorum < min_quorum
        {
            return Err(StdError::generic_err(format!(
                "The required quorum for a proposal cannot be lower than {}% or higher than {}%",
                min_quorum * Decimal::from_ratio(100u8, 1u8),
                max_quorum * Decimal::from_ratio(100u8, 1u8)
            )));
        }

        if !DELAY_INTERVAL.contains(&self.proposal_effective_delay) {
            return Err(StdError::generic_err(format!(
                "The effective delay for a proposal cannot be lower than {} or higher than {}",
                DELAY_INTERVAL.start(),
                DELAY_INTERVAL.end()
            )));
        }

        if !EXPIRATION_PERIOD_INTERVAL.contains(&self.proposal_expiration_period) {
            return Err(StdError::generic_err(format!(
                "The expiration period for a proposal cannot be lower than {} or higher than {}",
                EXPIRATION_PERIOD_INTERVAL.start(),
                EXPIRATION_PERIOD_INTERVAL.end()
            )));
        }

        if !VOTING_PERIOD_INTERVAL.contains(&self.proposal_voting_period) {
            return Err(StdError::generic_err(format!(
                "The voting period for a proposal should be more than {} or less than {} blocks.",
                VOTING_PERIOD_INTERVAL.start(),
                VOTING_PERIOD_INTERVAL.end()
            )));
        }

        if !DEPOSIT_INTERVAL.contains(&self.proposal_required_deposit.u128()) {
            return Err(StdError::generic_err(format!(
                "The required deposit for a proposal cannot be lower than {} or higher than {}",
                DEPOSIT_INTERVAL.start(),
                DEPOSIT_INTERVAL.end()
            )));
        }

        Ok(())
    }
}

#[cw_serde]
pub struct UpdateConfig {
    pub proposal_voting_period: Option<u64>,
    pub proposal_effective_delay: Option<u64>,
    pub proposal_expiration_period: Option<u64>,
    pub proposal_required_deposit: Option<u128>,
    pub proposal_required_quorum: Option<String>,
    pub proposal_required_threshold: Option<String>,
}