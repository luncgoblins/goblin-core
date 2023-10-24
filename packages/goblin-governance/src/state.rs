use crate::config::Config;
use crate::proposal::Proposal;
use cosmwasm_std::{Uint64, Coin, Storage, StdResult, StdError};
use cw_storage_plus::{Item, Map};

/// Stores the config for the Assembly contract
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the global state for the Assembly contract
pub const PROPOSAL_COUNT: Item<Uint64> = Item::new("proposal_count");

/// This is a map that contains information about all proposals
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");

pub fn get_gov_token_denom( store: &dyn Storage ) -> StdResult<String>{

    Ok(CONFIG.load(store)?.gov_token_denom)

}

pub fn validate_funds(store: &dyn Storage, coins: Vec<Coin>) -> StdResult<Coin> {
    
    let gov_denom = get_gov_token_denom(store)?;
    
    if coins.len() != 1 {
        return Err(StdError::GenericErr { msg: String::from("funds shall only contain one denom") });
    }

    if coins[0].denom != gov_denom {
        return Err(StdError::GenericErr { msg: String::from("funds shall only contain valid denom") });
    }

    return Ok(coins[0].clone());

}