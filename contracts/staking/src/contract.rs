use cosmwasm_std::{
    entry_point, Env, MessageInfo, 
    Deps, DepsMut, StdResult, Response, QueryResponse, to_binary, Uint128, BankMsg, Coin, Addr
};
use goblin_staking::state::{
    get_staking_amount_from_funds, insert_stake,
    remove_stake, add_claim, calculate_release_date,
    remove_released_claims, get_staking_denom, get_partitioned_claims,
    STAKES, TOTAL_STAKED, CONFIG,
};
use goblin_staking::query::{
    QueryMsg, QueryClaimsResp, QueryStakersResp
};
use goblin_staking::msg::{ExecuteMsg, InstantiateMsg};
use goblin_staking::err::ContractError;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    
    CONFIG.save(deps.storage, &msg.config)?;
    Ok(Response::new())

}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    
    match msg {
        ExecuteMsg::Stake {  } => {
            execute_stake(deps, env, info)
        },
        ExecuteMsg::Unstake { amount } => {
            execute_unstake(deps, env, info, amount)
        },
        ExecuteMsg::Withdraw { } => {
            execute_withdraw(deps, env, info)
        }
    }

}

pub fn execute_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,    
) -> Result<Response, ContractError>{

    let amount = get_staking_amount_from_funds(deps.storage, info.funds)?;
    if amount.is_zero() {
        return Err(ContractError::InsufficientFunds{});
    }
    insert_stake(deps.storage, info.sender, amount, env.block.height)?;
    Ok(Response::new())

}

pub fn execute_unstake(  
    deps: DepsMut,
    env: Env,
    info: MessageInfo, 
    amount: Uint128,
) -> Result<Response, ContractError>{

    let release_date = calculate_release_date(deps.storage, env.block.time)?;
    remove_stake(deps.storage, info.sender.clone(), amount, env.block.height)?;
    add_claim(deps.storage, info.sender, amount, release_date)?;
    Ok(Response::new())

}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {

    let denom = get_staking_denom(deps.storage)?;
    let release_amnt = remove_released_claims(
        deps.storage, 
        info.sender.clone(), 
        env.block.time
    )?;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![ Coin{
            denom: denom,
            amount: release_amnt,
        }]
    };
    
    Ok(Response::new().add_message(msg))

}

#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> StdResult<QueryResponse> {

    match msg {
        QueryMsg::Stakers {  } => todo!(),
        QueryMsg::Staked { address } => query_staked_per_address(deps, env, address),
        QueryMsg::StakedAt { address, height } => query_staked_at_per_address(deps, env, address, height),
        QueryMsg::TotalStaked {  } => query_total_staked(deps, env),
        QueryMsg::TotalStakedAt { height } => query_total_staked_at(deps, env, height),
        QueryMsg::Claims { address } => query_claims(deps, env, address),
    }

}

pub fn query_all_stakers(
    deps: Deps,
    env: Env,
) -> StdResult<QueryResponse> {

    let stakers = STAKES.keys(
        deps.storage, 
        None, 
        None,
        cosmwasm_std::Order::Descending,
    ).filter_map(|result| {
        match result {
            Ok(addr) => Some(addr.to_string()),
            Err(_) => None,
        }
    }).collect::<Vec<_>>();

    let resp = QueryStakersResp{
        stakers: stakers,
    };

    Ok(to_binary(&resp)?)

}

pub fn query_staked_per_address(
    deps: Deps,
    env: Env,
    address: Addr,
) -> StdResult<QueryResponse> {

    let height = env.block.height;
    return query_staked_at_per_address(deps, env, address, height)

}

pub fn query_staked_at_per_address(
    deps: Deps,
    env: Env,
    address: Addr,
    height: u64,
) -> StdResult<QueryResponse> {

    let resp = STAKES.may_load_at_height(deps.storage, address, height)?.unwrap_or(Uint128::from(0u32));
    Ok(to_binary(&resp)?)

}

pub fn query_total_staked(
    deps: Deps,
    env: Env,
) -> StdResult<QueryResponse> {

    let height = env.block.height;
    return query_total_staked_at(deps, env, height)

}

pub fn query_total_staked_at(
    deps: Deps,
    env: Env,
    height: u64,
) -> StdResult<QueryResponse> {

    let resp = TOTAL_STAKED.may_load_at_height(deps.storage, height)?.unwrap_or(Uint128::from(0u32));
    Ok(to_binary(&resp)?)

}

pub fn query_claims(
    deps: Deps,
    env: Env,
    address: Addr,
) -> StdResult<QueryResponse> {

    let (released_claims, unreleased_claims) = get_partitioned_claims(
        deps.storage,
        address, 
        env.block.time,
    )?;

    let total_unreleased = unreleased_claims.total()?;
    let total_released = released_claims.total()?;
    let total = total_unreleased.checked_add(total_released)?;
    
    let resp = QueryClaimsResp {
        released: released_claims,
        total_released: total_released,
        locked: unreleased_claims,
        total_locked: total_unreleased,
        total: total,
    };
    
    Ok(to_binary(&resp)?)

}