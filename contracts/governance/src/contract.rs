use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, CosmosMsg, Decimal,
    Deps, DepsMut, Env, MessageInfo, Order, Coin,
    Response, StdResult, Uint128, Uint64, WasmMsg, BankMsg,
};
use cw_storage_plus::Bound;
use goblin_governance::{
    msg::{InstantiateMsg, ExecuteMsg},
    config::{Config, UpdateConfig},
    state::{CONFIG, PROPOSAL_COUNT, PROPOSALS, validate_funds},
    err::ContractError, proposal::{Proposal, ProposalStatus, ProposalVoteOption},
    query::{QueryMsg, ProposalListResponse, ProposalVotesResponse},
};
use std::str::FromStr;
use goblin_staking::query::QueryMsg as StakingQueryMsg;


// Default pagination constants
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;
const DEFAULT_VOTERS_LIMIT: u32 = 100;
const MAX_VOTERS_LIMIT: u32 = 250;

/// Creates a new contract with the specified parameters in the `msg` variable.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let config = Config {
        staking_addr: deps.api.addr_validate(&msg.staking_addr)?,
        vesting_addr: deps.api.addr_validate(&msg.vesting_addr)?,
        gov_token_denom: msg.gov_token_denom,
        proposal_voting_period: msg.proposal_voting_period,
        proposal_effective_delay: msg.proposal_effective_delay,
        proposal_expiration_period: msg.proposal_expiration_period,
        proposal_required_deposit: msg.proposal_required_deposit,
        proposal_required_quorum: Decimal::from_str(&msg.proposal_required_quorum)?,
        proposal_required_threshold: Decimal::from_str(&msg.proposal_required_threshold)?,
    };

    config.validate()?;
    CONFIG.save(deps.storage, &config)?;
    PROPOSAL_COUNT.save(deps.storage, &Uint64::zero())?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SubmitProposal {
            title,
            description,
            link,
            messages,
        } => submit_proposal(deps, env, info, title, description, link, messages),
        ExecuteMsg::CastVote { proposal_id, vote } => cast_vote(deps, env, info, proposal_id, vote),
        ExecuteMsg::EndProposal { proposal_id } => end_proposal(deps, env, proposal_id),
        ExecuteMsg::ExecuteProposal { proposal_id } => execute_proposal(deps, env, proposal_id),
        ExecuteMsg::CheckMessages { messages } => check_messages(env, messages),
        ExecuteMsg::CheckMessagesPassed {} => Err(ContractError::MessagesCheckPassed {}),
        ExecuteMsg::RemoveCompletedProposal { proposal_id } => {
            remove_completed_proposal(deps, env, proposal_id)
        }
        ExecuteMsg::UpdateConfig(config) => update_config(deps, env, info, config),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn submit_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    link: Option<String>,
    messages: Option<Vec<CosmosMsg>>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // check the deposit
    let deposit_coin = validate_funds(deps.storage, info.funds)?;
    let deposit_amount = deposit_coin.amount; 
    if deposit_amount < config.proposal_required_deposit {
        return Err(ContractError::InsufficientDeposit {});
    }

    // Update the proposal count
    let count = PROPOSAL_COUNT.update(deps.storage, |c| -> StdResult<_> {
        Ok(c.checked_add(Uint64::new(1))?)
    })?;

    let proposal = Proposal {
        proposal_id: count,
        submitter: info.sender.clone(),
        status: ProposalStatus::Active,
        yes_power: Uint128::zero(),
        no_power: Uint128::zero(),
        yes_voters: Vec::new(),
        no_voters: Vec::new(),
        start_block: env.block.height,
        start_time: env.block.time.seconds(),
        end_block: env.block.height + config.proposal_voting_period,
        delayed_end_block: env.block.height
            + config.proposal_voting_period
            + config.proposal_effective_delay,
        expiration_block: env.block.height
            + config.proposal_voting_period
            + config.proposal_effective_delay
            + config.proposal_expiration_period,
        title,
        description,
        link,
        messages,
        deposit_amount,
    };

    proposal.validate()?;
    PROPOSALS.save(deps.storage, count.u64(), &proposal)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "submit_proposal"),
        attr("submitter", info.sender),
        attr("proposal_id", count),
        attr(
            "proposal_end_height",
            (env.block.height + config.proposal_voting_period).to_string(),
        ),
    ]))
}

pub fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    vote_option: ProposalVoteOption,
) -> Result<Response, ContractError> {
    let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;

    if proposal.status != ProposalStatus::Active {
        return Err(ContractError::ProposalNotActive {});
    }

    // submitter of the prop cannot vote?
    if proposal.submitter == info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if env.block.height > proposal.end_block {
        return Err(ContractError::VotingPeriodEnded {});
    }

    if proposal.yes_voters.contains(&info.sender) || proposal.no_voters.contains(&info.sender) {
        return Err(ContractError::UserAlreadyVoted {});
    }

    let voting_power = calc_voting_power(deps.as_ref(), info.sender.to_string(), &proposal)?;

    if voting_power.is_zero() {
        return Err(ContractError::NoVotingPower {});
    }

    match vote_option {
        ProposalVoteOption::For => {
            proposal.yes_power = proposal.yes_power.checked_add(voting_power)?;
            proposal.yes_voters.push(info.sender.clone());
        }
        ProposalVoteOption::Against => {
            proposal.no_power = proposal.no_power.checked_add(voting_power)?;
            proposal.no_voters.push(info.sender.clone());
        }
    };

    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "cast_vote"),
        attr("proposal_id", proposal_id.to_string()),
        attr("voter", &info.sender),
        attr("vote", vote_option.to_string()),
        attr("voting_power", voting_power),
    ]))
}

pub fn end_proposal(deps: DepsMut, env: Env, proposal_id: u64) -> Result<Response, ContractError> {
    let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;

    if proposal.status != ProposalStatus::Active {
        return Err(ContractError::ProposalNotActive {});
    }

    if env.block.height <= proposal.end_block {
        return Err(ContractError::VotingPeriodNotEnded {});
    }

    let config = CONFIG.load(deps.storage)?;

    let for_votes = proposal.yes_power;
    let against_votes = proposal.no_power;
    let total_votes = for_votes + against_votes;

    let total_voting_power = calc_total_voting_power_at(deps.as_ref(), &proposal)?;

    let mut proposal_quorum: Decimal = Decimal::zero();
    let mut proposal_threshold: Decimal = Decimal::zero();

    if !total_voting_power.is_zero() {
        proposal_quorum = Decimal::from_ratio(total_votes, total_voting_power);
    }

    if !total_votes.is_zero() {
        proposal_threshold = Decimal::from_ratio(for_votes, total_votes);
    }

    // Determine the proposal result
    proposal.status = if proposal_quorum >= config.proposal_required_quorum
        && proposal_threshold > config.proposal_required_threshold
    {
        ProposalStatus::Passed
    } else {
        ProposalStatus::Rejected
    };

    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    let response = Response::new()
        .add_attributes(vec![
            attr("action", "end_proposal"),
            attr("proposal_id", proposal_id.to_string()),
            attr("proposal_result", proposal.status.to_string()),
        ])
        .add_message(BankMsg::Send{
            to_address: proposal.submitter.into_string(),
            amount: vec![Coin{ 
                denom: config.gov_token_denom, 
                amount: proposal.deposit_amount,
            }],
        });

    Ok(response)
}

/// Executes a successful proposal by id.
pub fn execute_proposal(
    deps: DepsMut,
    env: Env,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;

    if proposal.status != ProposalStatus::Passed {
        return Err(ContractError::ProposalNotPassed {});
    }

    if env.block.height < proposal.delayed_end_block {
        return Err(ContractError::ProposalDelayNotEnded {});
    }

    if env.block.height > proposal.expiration_block {
        return Err(ContractError::ExecuteProposalExpired {});
    }

    let messages;
    
    // mark as executed and unwrap proposal messages
    proposal.status = ProposalStatus::Executed;
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;
    messages = proposal.messages.unwrap_or_default();

    Ok(Response::new()
        .add_attribute("action", "execute_proposal")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_messages(messages))
}

/// Checks that proposal messages are correct.
pub fn check_messages(env: Env, mut messages: Vec<CosmosMsg>) -> Result<Response, ContractError> {
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::CheckMessagesPassed {})?,
        funds: vec![],
    }));

    Ok(Response::new()
        .add_attribute("action", "check_messages")
        .add_messages(messages))
}

/// Removes an expired or rejected proposal from the general proposal list.
pub fn remove_completed_proposal(
    deps: DepsMut,
    env: Env,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;

    if env.block.height
        > (proposal.end_block + config.proposal_effective_delay + config.proposal_expiration_period)
    {
        proposal.status = ProposalStatus::Expired;
    }

    if proposal.status != ProposalStatus::Expired && proposal.status != ProposalStatus::Rejected {
        return Err(ContractError::ProposalNotCompleted {});
    }

    PROPOSALS.remove(deps.storage, proposal_id);

    Ok(Response::new()
        .add_attribute("action", "remove_completed_proposal")
        .add_attribute("proposal_id", proposal_id.to_string()))
}

pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    updated_config: Box<UpdateConfig>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // Only the Assembly is allowed to update its own parameters (through a successful proposal)
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(proposal_voting_period) = updated_config.proposal_voting_period {
        config.proposal_voting_period = proposal_voting_period;
    }

    if let Some(proposal_effective_delay) = updated_config.proposal_effective_delay {
        config.proposal_effective_delay = proposal_effective_delay;
    }

    if let Some(proposal_expiration_period) = updated_config.proposal_expiration_period {
        config.proposal_expiration_period = proposal_expiration_period;
    }

    if let Some(proposal_required_deposit) = updated_config.proposal_required_deposit {
        config.proposal_required_deposit = Uint128::from(proposal_required_deposit);
    }

    if let Some(proposal_required_quorum) = updated_config.proposal_required_quorum {
        config.proposal_required_quorum = Decimal::from_str(&proposal_required_quorum)?;
    }

    if let Some(proposal_required_threshold) = updated_config.proposal_required_threshold {
        config.proposal_required_threshold = Decimal::from_str(&proposal_required_threshold)?;
    }

    config.validate()?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::Proposals { start, limit } => to_binary(&query_proposals(deps, start, limit)?),
        QueryMsg::Proposal { proposal_id } => {
            to_binary(&PROPOSALS.load(deps.storage, proposal_id)?)
        }
        QueryMsg::ProposalVotes { proposal_id } => {
            to_binary(&query_proposal_votes(deps, proposal_id)?)
        }
        QueryMsg::UserVotingPower { user, proposal_id } => {
            let proposal = PROPOSALS.load(deps.storage, proposal_id)?;

            deps.api.addr_validate(&user)?;

            to_binary(&calc_voting_power(deps, user, &proposal)?)
        }
        QueryMsg::TotalVotingPower { proposal_id } => {
            let proposal = PROPOSALS.load(deps.storage, proposal_id)?;
            to_binary(&calc_total_voting_power_at(deps, &proposal)?)
        }
        QueryMsg::ProposalVoters {
            proposal_id,
            vote_option,
            start,
            limit,
        } => to_binary(&query_proposal_voters(
            deps,
            proposal_id,
            vote_option,
            start,
            limit,
        )?),
    }
}

/// Returns the current proposal list.
pub fn query_proposals(
    deps: Deps,
    start: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProposalListResponse> {
    let proposal_count = PROPOSAL_COUNT.load(deps.storage)?;

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start.map(Bound::inclusive);

    let proposal_list = PROPOSALS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(ProposalListResponse {
        proposal_count,
        proposal_list,
    })
}

/// Returns proposal's voters.
pub fn query_proposal_voters(
    deps: Deps,
    proposal_id: u64,
    vote_option: ProposalVoteOption,
    start: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<Addr>> {
    let limit = limit.unwrap_or(DEFAULT_VOTERS_LIMIT).min(MAX_VOTERS_LIMIT);
    let start = start.unwrap_or_default();

    let proposal = PROPOSALS.load(deps.storage, proposal_id)?;

    let voters = match vote_option {
        ProposalVoteOption::For => proposal.yes_voters,
        ProposalVoteOption::Against => proposal.no_voters,
    };

    Ok(voters
        .iter()
        .skip(start as usize)
        .take(limit as usize)
        .cloned()
        .collect())
}

/// Returns proposal votes stored in the [`ProposalVotesResponse`] structure.
pub fn query_proposal_votes(deps: Deps, proposal_id: u64) -> StdResult<ProposalVotesResponse> {
    let proposal = PROPOSALS.load(deps.storage, proposal_id)?;

    Ok(ProposalVotesResponse {
        proposal_id,
        for_power: proposal.yes_power,
        against_power: proposal.no_power,
    })
}

pub fn calc_voting_power(deps: Deps, sender: String, proposal: &Proposal) -> StdResult<Uint128> {
    let config = CONFIG.load(deps.storage)?;

    // The staked balance of the user is his voting power
    let staked_user_balance: Uint128 = deps.querier.query_wasm_smart(
        config.staking_addr,
        &StakingQueryMsg::StakedAt {
            address: deps.api.addr_validate(&sender.clone())?,
            height: proposal.start_block - 1,
        },
    )?;
    Ok(staked_user_balance)

}

pub fn calc_total_voting_power_at(deps: Deps, proposal: &Proposal) -> StdResult<Uint128> {
    let config = CONFIG.load(deps.storage)?;
    
    // The staked total balance
    let staked_total_balance: Uint128 = deps.querier.query_wasm_smart(
        config.staking_addr,
        &StakingQueryMsg::TotalStakedAt {
            height: proposal.start_block - 1,
        },
    )?;
    Ok(staked_total_balance)

}