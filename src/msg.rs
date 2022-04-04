use cosmwasm_std::{Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub token_addr: String,
    pub stable_denom: String,
    pub admin: Option<String>,
    pub team_wallet: Option<String>,
    pub start_time: Uint128,
    pub lock_time: Uint128,
    pub vesting_time: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserUpdateData {
    pub user_addr: String,
    pub allocation: Uint128,
    pub refunded: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Deposit {},
    CanDeposit(bool),
    Update(Vec<UserUpdateData>),
    UpdateToken(String),
    EnableWithdraw {},
    Claim {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    List {},
    ClaimableAmount { wallet: String },
    Investor { wallet: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    StakeVotingTokens {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: String,
    pub token_addr: String,
    pub team_wallet: String,
    pub stable_denom: String,
    pub start_time: Uint128,
    pub lock_time: Uint128,
    pub vesting_time: Uint128,

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InvestorResponse {
    pub total_deposited: Uint128,
    pub allocation: Uint128,
    pub refunded: Uint128,
    pub is_refunded: bool,
    pub deposit_history: Vec<DepositInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositInfo {
    pub date: Timestamp,
    pub amount: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableAmountResponse {
    pub lock_time_left: u64, //in seconds
    pub tokens_allocated: String,
    pub token_avaiable_to_claim: String,
    pub returned_ust: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WalletInfo {
    pub wallet: String,
    pub total: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListResponse {
    pub investors: Vec<WalletInfo>,
}
