use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Map, Item};
use ariel::types::DepositDirection;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositRecord {
    pub ts: u64,
    pub record_id: u64,
    pub user: Addr,
    pub direction: DepositDirection,
    pub collateral_before: Uint128,
    pub cumulative_deposits_before: Uint128,
    pub amount: u64,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositInfo {
    pub len: u64,
}

pub const DEPOSIT_HISTORY: Map<(Addr, u64),  DepositRecord> = Map::new("deposit_history");
pub const DEPOSIT_HISTORY_INFO: Item<DepositInfo> = Item::new("deposit_history_info");
