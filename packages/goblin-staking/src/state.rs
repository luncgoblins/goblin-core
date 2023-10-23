use cosmwasm_std::{Addr, Uint128, Coin, Storage, StdResult, Timestamp};
use cw_storage_plus::{SnapshotMap, Strategy, Item, Map, SnapshotItem};
use crate::{claim::{Claim, Claims}, config::Config};

pub const STAKES: SnapshotMap<Addr, Uint128> = SnapshotMap::new(
    "stakes",
    "stakes__check",
    "stakes__change",
    Strategy::EveryBlock,
);

pub const TOTAL_STAKED: SnapshotItem<Uint128> = SnapshotItem::new(
    "total_staked",
    "total_staked__check",
    "total_staked__change",
    Strategy::EveryBlock,
);

pub const CLAIMS: Map<Addr, Claims> = Map::new(
    "claims"
);

pub const CONFIG: Item<Config> = Item::new(
    "config",
);

// input can be a list of coins. output the total
// amount of staking tokens from the funds input
pub fn get_staking_amount_from_funds(
    store: &dyn Storage,
    funds: Vec<Coin>,
) -> StdResult<Uint128> {
    
    let staking_denom = get_staking_denom(store)?;
    let amount: StdResult<Uint128> = funds
        .into_iter()
        .fold(Ok(Uint128::from(0u16)), |cum: StdResult<Uint128>, coin: Coin| -> StdResult<Uint128>  {
            let mut ret = cum?;
            if coin.denom == staking_denom {
                ret = ret.checked_add(coin.amount)?
            }
            Ok(ret)
        });
    Ok(amount?)
       
}

// get the staking denom from the config
pub fn get_staking_denom(
    store: &dyn Storage
) -> StdResult<String> {

    Ok(CONFIG.load(store)?.denom)

}

// insert new stake for the given address
pub fn insert_stake(
    store: &mut dyn Storage,
    address: Addr,
    amount: Uint128,
    height: u64,
) -> StdResult<()> {

    let total_stake = TOTAL_STAKED.load(store)?;
    let stake = STAKES
        .load(store, address.clone())
        .unwrap_or(Uint128::new(0u128));
    let new_stake = stake.checked_add(amount)?;
    let new_total_stake = total_stake.checked_add(amount)?;
    STAKES.save(store, address, &new_stake, height)?;
    TOTAL_STAKED.save(store, &new_total_stake, height)?;
    return Ok(())

}

// remove stake from the given address
// turn it into an to-be-released claim
pub fn remove_stake(
    store: &mut dyn Storage,
    address: Addr,
    amount: Uint128,
    height: u64,
) -> StdResult<()> {

    let total_stake = TOTAL_STAKED.load(store)?;
    let stake = STAKES
        .load(store, address.clone())
        .unwrap_or(Uint128::new(0u128));
    let new_amount = stake.checked_sub(amount)?;
    let new_total_stake = total_stake.checked_sub(amount)?;
    STAKES.save(store, address, &new_amount, height)?;
    TOTAL_STAKED.save(store, &new_total_stake, height)?;
    return Ok(())

}

pub fn get_total_staked_per_address_at(
    store: &dyn Storage,
    address: Addr,
    height: u64,
) -> StdResult<Uint128> {

    Ok(
        STAKES.may_load_at_height(store, address, height)?
            .unwrap_or(Uint128::from(0u64))
    )

}

// add a claim to the list of claims
// for a particular user
pub fn add_claim(
    store: &mut dyn Storage,
    address: Addr,
    amount: Uint128,
    release: Timestamp
) -> StdResult<()> {

    let mut claims = CLAIMS.load(store, address.clone()).unwrap_or(Claims::new());
    let new_claim = Claim{
        amount: amount,
        release: release,
    };
    claims.add(new_claim);
    CLAIMS.save(store, address, &claims)

}

// get the 
pub fn get_lock_period(
    store: &dyn Storage
) -> StdResult<u64> {

    Ok(CONFIG.load(store)?.lock_period)

}

pub fn calculate_release_date(
    store: &dyn Storage,
    now: Timestamp,
) -> StdResult<Timestamp> {

    let period = get_lock_period(store)?;
    let then = now.plus_seconds(period);
    Ok(then)

}

pub fn get_partitioned_claims(
    store: &dyn Storage,
    address: Addr,
    now: Timestamp,
) -> StdResult<(Claims, Claims)> {

    Ok( CLAIMS
        .load(store, address.clone())?
        .into_iter()
        .partition(|c| c.is_released(now))
    )

}

// remove all claims of owner "address" that are in the state of
// having been released as of "now"
pub fn remove_released_claims(
    store: &mut dyn Storage,
    address: Addr,
    now: Timestamp,
) -> StdResult<Uint128> {

    let (released_claims, unreleased_claims) = get_partitioned_claims(store, address.clone(), now)?;
    
    // rewrite only unreleased claims
    CLAIMS.save(store, address, &unreleased_claims.into())?;
    
    // return the amount of claims that have been released
    released_claims.total()

}