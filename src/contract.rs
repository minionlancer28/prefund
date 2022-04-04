use cosmwasm_std::{
    attr, coin, entry_point, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::msg::{
    ClaimableAmountResponse, ConfigResponse, DepositInfo, ExecuteMsg, InstantiateMsg,
    InvestorResponse, ListResponse, QueryMsg, UserUpdateData, WalletInfo,
};
use crate::state::{
    Config, DepositData, Status, UserData, CONFIG, DEPOSIT,  RELEASED_INFO,
    RETURNED_REFUNDED_INFO, STATUS, USER_DATA,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            token_addr: deps.api.addr_canonicalize(msg.token_addr.as_str())?,
            stable_denom: msg.stable_denom,
            start_time: msg.start_time.u128() as u64,
            lock_time: msg.lock_time.u128() as u64,
            vesting_time: msg.vesting_time.u128() as u64,
            admin: if let Some(admin) = msg.admin {
                deps.api.addr_canonicalize(&admin)?
            } else {
                deps.api.addr_canonicalize(info.sender.as_str())?
            },
            team_wallet: if let Some(team_wallet) = msg.team_wallet {
                deps.api.addr_canonicalize(&team_wallet)?
            } else {
                deps.api.addr_canonicalize(info.sender.as_str())?
            },

        },
    )?;

    STATUS.save(
        deps.storage,
        &Status {
            can_deposit: false,
            can_withdraw: false,
        },
    )?;
    Ok(Response::new().add_attributes(vec![attr("action", "instantiate")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Deposit {} => deposit(deps, env, info),
        ExecuteMsg::CanDeposit(can_deposit) => enable_deposit(deps, env, info, can_deposit),
        ExecuteMsg::EnableWithdraw {} => enable_withdraw(deps, env, info),
        ExecuteMsg::Update(user_data_list) => update(deps, env, info, user_data_list),
        ExecuteMsg::UpdateToken(token_addr) => update_token(deps, env, info, token_addr),
        ExecuteMsg::Claim {} => claim(deps, env, info),
    }
}
fn deposit(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let status = STATUS.load(deps.storage)?;
    if !status.can_deposit {
        return Err(StdError::generic_err("User is not able to deposit"));
    }
    if info.funds.len() > 1usize {
        return Err(StdError::generic_err(
            "More than one coin is sent; only one asset is supported",
        ));
    }
    let payment = info
        .funds
        .iter()
        .find(|x| x.denom == config.stable_denom && x.amount > Uint128::zero())
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "No {} assets are provided to deposit",
                config.stable_denom.clone()
            ))
        })?;

    let deposit = DEPOSIT.may_load(deps.storage, &info.sender)?;
    let deposit_data = if let Some(mut deposit_data) = deposit {
        deposit_data.total_deposited += payment.amount;
        deposit_data.deposit_history.push(DepositInfo {
            date: env.block.time,
            amount: payment.amount,
        });
        deposit_data
    } else {
        DepositData {
            total_deposited: payment.amount,
            deposit_history: vec![DepositInfo {
                date: env.block.time,
                amount: payment.amount,
            }],
        }
    };

    DEPOSIT.save(deps.storage, &info.sender, &deposit_data)?;
    Ok(Response::new().add_attributes(vec![
        attr("action", "deposit"),
        attr("amount", payment.amount),
    ]))
}

fn enable_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    can_deposit: bool,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }
    let mut status = STATUS.load(deps.storage)?;
    status.can_deposit = can_deposit;
    STATUS.save(deps.storage, &status)?;
    Ok(Response::new().add_attributes(vec![
        attr("action", "enable_deposit"),
        attr("can_deposit", can_deposit.to_string()),
    ]))
}

fn enable_withdraw(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }
    let mut status = STATUS.load(deps.storage)?;
    if !status.can_withdraw {
        status.can_withdraw = true;
        STATUS.save(deps.storage, &status)?;
    }

    Ok(Response::new().add_attributes(vec![
        attr("action", "enable_withdraw"),
        attr("can_withdraw", status.can_withdraw.to_string()),
    ]))
}
fn update_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_addr: String,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }
    config.token_addr = deps.api.addr_canonicalize(&token_addr)?;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attributes(vec![
        attr("atcion", " update_token"),
        attr("token_addr", token_addr),
    ]))
}

fn update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    user_data_list: Vec<UserUpdateData>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }
    let mut transfer_amount = Uint128::zero();
    for user_data in user_data_list.into_iter() {
        let data = UserData {
            allocation: user_data.allocation,
            refunded: user_data.refunded,
        };
        let user_addr = deps.api.addr_validate(&user_data.user_addr)?;
        USER_DATA.save(deps.storage, &user_addr, &data)?;
        let deposit_data = DEPOSIT.may_load(deps.storage, &user_addr)?;
        if let Some(deposit_data) = deposit_data {
            if deposit_data.total_deposited > data.refunded {
                transfer_amount += deposit_data.total_deposited - data.refunded;
            }
        }
    }
    let mut msgs = vec![];
    if transfer_amount > Uint128::zero() {
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: deps.api.addr_humanize(&config.team_wallet)?.to_string(),
            amount: vec![coin(transfer_amount.u128(), config.stable_denom)],
        }));
    }
    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        attr("action", "update"),
        attr("transfer_amount", transfer_amount.to_string()),
    ]))
}

fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let status = STATUS.load(deps.storage)?;
    if !status.can_withdraw {
        return Err(StdError::generic_err("do not allow to claim"));
    }
    let config = CONFIG.load(deps.storage)?;

    let returned_refunded_info = RETURNED_REFUNDED_INFO.may_load(deps.storage, &info.sender)?;
    let user_data = USER_DATA.load(deps.storage, &info.sender)?;
    let mut msgs = vec![];
    let mut return_refunded_amount = Uint128::zero();

    if user_data.refunded > Uint128::zero() && returned_refunded_info.is_none() {
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin(user_data.refunded.u128(), config.stable_denom)],
        }));
        RETURNED_REFUNDED_INFO.save(deps.storage, &info.sender, &user_data.refunded)?;
        return_refunded_amount = user_data.refunded.clone();
    }

    let start_time = config.start_time;
    let lock_time = config.lock_time;
    let vesting_time = config.vesting_time;
    let current_time = env.block.time.seconds();
    if start_time + lock_time > current_time {
        if msgs.len() > 0 {
            // claim UST
            return Ok(Response::new().add_messages(msgs).add_attributes(vec![
                attr("action", "claim"),
                attr("refunded_amount", return_refunded_amount.to_string()),
            ]));
        } else {
            return Err(StdError::generic_err("locking funds"));
        }
    }
    let passed_time = current_time - start_time - lock_time;

    let mut claimable_allocation_amount=   if passed_time > vesting_time {
        user_data.allocation
    }else{
        user_data
            .allocation
            .multiply_ratio(passed_time, vesting_time)
    };

    let released_info = RELEASED_INFO.may_load(deps.storage, &info.sender)?;
    let released_amount = if let Some(released_amount) = released_info {
        released_amount
    } else {
        Uint128::zero()
    };

    RELEASED_INFO.save(deps.storage, &info.sender, &claimable_allocation_amount)?;

    claimable_allocation_amount -= released_amount;

    if claimable_allocation_amount > Uint128::zero() {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.token_addr)?.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: claimable_allocation_amount,
            })?,
            funds: vec![],
        }));
    }
    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        attr("action", "claim"),
        attr("claim_amount", claimable_allocation_amount.to_string()),
        attr(
            "returned_refunded_amount",
            return_refunded_amount.to_string(),
        ),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::Investor { wallet } => Ok(to_binary(&query_investor(deps, wallet)?)?),
        QueryMsg::List {} => Ok(to_binary(&query_list(deps)?)?),
        QueryMsg::ClaimableAmount { wallet } => {
            Ok(to_binary(&query_claimable_amount(deps, env, wallet)?)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        admin: deps.api.addr_humanize(&config.admin)?.to_string(),
        token_addr: deps.api.addr_humanize(&config.token_addr)?.to_string(),
        stable_denom: config.stable_denom,
        team_wallet: deps.api.addr_humanize(&config.team_wallet)?.to_string(),
        start_time: Uint128::from(config.start_time),
        lock_time: Uint128::from(config.lock_time),
        vesting_time: Uint128::from(config.vesting_time),
    };
    Ok(resp)
}

fn query_investor(deps: Deps, wallet: String) -> StdResult<InvestorResponse> {
    let wallet_addr = deps.api.addr_validate(&wallet)?;
    let deposit_data = DEPOSIT.may_load(deps.storage, &wallet_addr)?;
    let deposit_data = if let Some(deposit_data) = deposit_data {
        deposit_data
    } else {
        DepositData {
            total_deposited: Uint128::zero(),
            deposit_history: vec![],
        }
    };

    let user_data = USER_DATA.may_load(deps.storage, &wallet_addr)?;
    let user_data = if let Some(user_data) = user_data {
        user_data
    } else {
        UserData {
            allocation: Uint128::zero(),
            refunded: Uint128::zero(),
        }
    };

    let refunded_info = RETURNED_REFUNDED_INFO.may_load(deps.storage, &wallet_addr)?;
    let is_refunded = refunded_info.is_some();

    Ok(InvestorResponse {
        total_deposited: deposit_data.total_deposited,
        allocation: user_data.allocation,
        refunded: user_data.refunded,
        is_refunded,
        deposit_history: deposit_data.deposit_history,
    })
}

fn query_claimable_amount(
    deps: Deps,
    env: Env,
    wallet: String,
) -> StdResult<ClaimableAmountResponse> {
    let wallet_addr = Addr::unchecked(wallet);
    let user_data = USER_DATA.may_load(deps.storage, &wallet_addr)?;
    let user_data = if let Some(user_data) = user_data {
        user_data
    } else {
        UserData {
            allocation: Uint128::zero(),
            refunded: Uint128::zero(),
        }
    };
    let config = CONFIG.load(deps.storage)?;
    let status = STATUS.load(deps.storage)?;

    if !status.can_withdraw {
        return Ok(ClaimableAmountResponse {
            lock_time_left: 0,
            tokens_allocated: user_data.allocation.to_string(),
            token_avaiable_to_claim: "0".to_string(),
            returned_ust: "0".to_string(),
        });
    }
    let refunded_amount = RETURNED_REFUNDED_INFO.may_load(deps.storage, &wallet_addr)?;
    let refunded_amount = if let Some(refunded_amount) = refunded_amount {
        refunded_amount
    } else {
        Uint128::zero()
    };
    let lock_time = config.lock_time;
    let start_time = config.start_time;
    let vesting_time = config.vesting_time;
    let current_time = env.block.time.seconds();
    if start_time + lock_time > current_time {
        return Ok(ClaimableAmountResponse {
            lock_time_left: start_time + lock_time - current_time,
            tokens_allocated: user_data.allocation.to_string(),
            token_avaiable_to_claim: "0".to_string(),
            returned_ust: refunded_amount.to_string(),
        });
    }
    let passed_time = current_time - start_time - lock_time;

    let mut claimable_allocation_amount: Uint128 = if passed_time > vesting_time {
        user_data.allocation
    }else{
        user_data
            .allocation
            .multiply_ratio(passed_time, vesting_time)
    };

    let released_info = RELEASED_INFO.may_load(deps.storage, &wallet_addr)?;
    let released_amount = if let Some(released_amount) = released_info {
        released_amount
    } else {
        Uint128::zero()
    };

    claimable_allocation_amount -= released_amount;

    Ok(ClaimableAmountResponse {
        lock_time_left: 0,
        tokens_allocated: user_data.allocation.to_string(),
        token_avaiable_to_claim: claimable_allocation_amount.to_string(),
        returned_ust: refunded_amount.to_string(),
    })
}
fn query_list(deps: Deps) -> StdResult<ListResponse> {
    let investors = DEPOSIT
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|v| v.ok())
        .map(|(k, v)| WalletInfo {
            wallet: Addr::unchecked(String::from_utf8(k).unwrap()).to_string(),
            total: v.total_deposited,
        })
        .collect();
    Ok(ListResponse { investors })
}
