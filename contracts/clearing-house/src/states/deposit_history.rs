use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositHistory {
    pub timestamp: u64,
    pub record_id: u64,
    pub user_address: Addr,
    pub direction: Direction,
    pub initial_collateral: Uint128,
    pub initial_deposit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Direction {
    Deposit,
    Withdraw
}
//TODO:: making the funding rate history map to composit key
pub const CONFIG: Map<u64, DepositHistory> = Map::new("funding_payment_history");
