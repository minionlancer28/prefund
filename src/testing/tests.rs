use crate::contract::{execute, instantiate, query};
use crate::msg::ExecuteMsg::{CanDeposit, Claim, Deposit, EnableWithdraw, Update};
use crate::msg::{DepositInfo, InstantiateMsg, InvestorResponse, QueryMsg, UserUpdateData};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    attr, coin, from_binary, to_binary, BankMsg, CosmosMsg, Response, StdError, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

#[test]
fn test_instantiate() {
    let owner = "creator";
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info(owner, &[]);
    let msg = InstantiateMsg {
        token_addr: "SAYVE_TOKEN".to_string(),
        stable_denom: "uusd".to_string(),
        admin: Some("ADMIN1".to_string()),
        team_wallet: Some("TEAM_WALLET".to_string()),
    };
    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_attributes(vec![attr("action", "instantiate")])
    )
}

#[test]
fn test_deposit() {
    let owner = "creator";
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info(owner, &[]);
    let init_msg = InstantiateMsg {
        token_addr: "SAYVE_TOKEN".to_string(),
        stable_denom: "uusd".to_string(),
        admin: Some("ADMIN1".to_string()),
        team_wallet: Some("TEAM_WALLET".to_string()),
    };
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

    let deposit_msg = Deposit {};
    let info = mock_info("USER1", &[coin(1000, "uusd")]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        deposit_msg.clone(),
    )
    .unwrap_err();
    assert_eq!(res, StdError::generic_err("User is not able to deposit"));

    //check deposit
    let can_deposit_msg = CanDeposit(true);
    let info = mock_info("ADMIN1", &[]);
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        can_deposit_msg.clone(),
    )
    .unwrap();

    let info = mock_info("USER1", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, deposit_msg.clone()).unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err("No uusd assets are provided to deposit")
    );

    let info = mock_info("USER1", &[coin(10000u128, "uusd")]);
    let res = execute(deps.as_mut(), env.clone(), info, deposit_msg.clone()).unwrap();
    assert_eq!(
        res,
        Response::new().add_attributes(vec![attr("action", "deposit"), attr("amount", "10000")])
    );
}

#[test]
fn test_claim() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("creator", &[]);
    let init_msg = InstantiateMsg {
        token_addr: "SAYVE_TOKEN".to_string(),
        stable_denom: "uusd".to_string(),
        admin: Some("ADMIN1".to_string()),
        team_wallet: Some("TEAM_WALLET".to_string()),
    };
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

    //check deposit
    let can_deposit_msg = CanDeposit(true);
    let info = mock_info("ADMIN1", &[]);
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        can_deposit_msg.clone(),
    )
    .unwrap();

    //can_deposit
    let can_deposit_msg = CanDeposit(true);
    let info = mock_info("ADMIN1", &[]);
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        can_deposit_msg.clone(),
    )
    .unwrap();

    //deposit
    let deposit_msg = Deposit {};
    let info = mock_info("USER1", &[coin(10000, "uusd")]);
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        deposit_msg.clone(),
    )
    .unwrap();

    //update
    let update_msg = Update(vec![UserUpdateData {
        user_addr: "USER1".to_string(),
        allocation: Uint128::from(10000u128),
        refunded: Uint128::from(1000u128),
    }]);
    let info = mock_info("ADMIN1", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), update_msg.clone()).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_messages(vec![CosmosMsg::Bank(BankMsg::Send {
                to_address: "TEAM_WALLET".to_string(),
                amount: vec![coin(9000, "uusd")]
            })])
            .add_attributes(vec![
                attr("action", "update"),
                attr("transfer_amount", "9000")
            ])
    );

    //start_deposit

    //can_withdraw

    //claim
    let can_withdraw = EnableWithdraw {};
    let info = mock_info("ADMIN1", &[]);
    let env = mock_env();
    let _res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        can_withdraw.clone(),
    )
    .unwrap();

    let claim_msg = Claim {};
    let info = mock_info("USER1", &[]);
    let mut env = mock_env();
    let time_stamp = env.clone().block.time.plus_seconds(26784000); //31 days later.

    env.block.time = time_stamp;

    let res = execute(deps.as_mut(), env.clone(), info.clone(), claim_msg.clone()).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_messages(vec![
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: "USER1".to_string(),
                    amount: vec![coin(1000u128, "uusd")]
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: "SAYVE_TOKEN".to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: "USER1".to_string(),
                        amount: Uint128::from(3333u128),
                    })
                    .unwrap(),
                    funds: vec![]
                }),
            ])
            .add_attributes(vec![
                attr("action", "claim"),
                attr("claim_amount", "3333"),
                attr("returned_refunded_amount", "1000"),
            ])
    );

    let time_stamp = env.clone().block.time.plus_seconds(86400); //1 days later.
    let mut env = env.clone();
    env.block.time = time_stamp;
    let res = execute(deps.as_mut(), env.clone(), info.clone(), claim_msg.clone()).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "SAYVE_TOKEN".to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "USER1".to_string(),
                    amount: Uint128::from(3333u128),
                })
                .unwrap(),
                funds: vec![]
            })])
            .add_attributes(vec![
                attr("action", "claim"),
                attr("claim_amount", "3333"),
                attr("returned_refunded_amount", "0")
            ])
    );

    // check query_investor
    let env = mock_env();
    let msg = QueryMsg::Investor {
        wallet: "USER1".to_string(),
    };
    let res: InvestorResponse =
        from_binary(&query(deps.as_ref(), env.clone(), msg).unwrap()).unwrap();
    assert_eq!(
        res,
        InvestorResponse {
            total_deposited: Uint128::from(10000u128),
            allocation: Uint128::from(10000u128),
            refunded: Uint128::from(1000u128),
            is_refunded: true,
            deposit_history: vec![DepositInfo {
                date: env.block.time,
                amount: Uint128::from(10000u128),
            }]
        }
    )
}
