use cosmwasm_std::{Addr, Uint128};
use serde::{Serialize, Deserialize};
use crate::claim::Claims;

#[derive(Serialize, Deserialize)]
pub enum QueryMsg {
    Stakers { },
    Staked { address: Addr },
    StakedAt { address: Addr, height: u64},
    TotalStaked {},
    TotalStakedAt { height: u64 },
    Claims { address: Addr },
}

#[derive(Serialize, Deserialize)]
pub struct QueryStakersResp {
    pub stakers: Vec<String>
}

#[derive(Serialize, Deserialize)]
pub struct QueryStakedResp {
    pub staked: Uint128
}

#[derive(Serialize, Deserialize)]
pub struct QueryClaimsResp {
    pub released:           Claims,
    pub total_released:     Uint128,
    pub locked:             Claims,
    pub total_locked:       Uint128,
    pub total:              Uint128,
}