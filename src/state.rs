use crate::msg::DepositInfo;
use cosmwasm_std::{Addr, CanonicalAddr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: CanonicalAddr,
    pub team_wallet: CanonicalAddr,
    pub token_addr: CanonicalAddr,
    pub stable_denom: String,
    pub start_time: u64,
    pub lock_time: u64,
    pub vesting_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Status {
    pub can_deposit: bool,
    pub can_withdraw: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositData {
    pub total_deposited: Uint128,
    pub deposit_history: Vec<DepositInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserData {
    pub allocation: Uint128,
    pub refunded: Uint128,
}
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATUS: Item<Status> = Item::new("status");

pub const DEPOSIT: Map<&Addr, DepositData> = Map::new("deposit_info");

// ( allocation, refunded)
pub const USER_DATA: Map<&Addr, UserData> = Map::new("user_data");

// allocation
pub const RELEASED_INFO: Map<&Addr, Uint128> = Map::new("released_info");
//refunded
pub const RETURNED_REFUNDED_INFO: Map<&Addr, Uint128> = Map::new("refunded_info");

// pub const LOCK_TIME: u64 = 2592000; // 30DAYS in seconds

// pub const VESTING_TIME: u64 = 7776000; //90DAYS in seconds
