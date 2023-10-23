use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::config::Config;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExecuteMsg {
    Stake{},
    Unstake{
        amount: Uint128,
    },
    Withdraw{},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub config: Config
}