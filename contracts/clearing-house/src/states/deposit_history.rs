use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use cw_storage_plus::{Map, Item};
use ariel::types::DepositDirection;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositRecord {
    pub ts: u64,
    pub record_id: u128,
    pub user: Addr,
    pub direction: DepositDirection,
    pub collateral_before: u128,
    pub cumulative_deposits_before: i128,
    pub amount: u64,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositInfo {
    pub len: u64,
}

pub const DepositHistory: Map<(u64,Addr),  DepositRecord> = Map::new("deposit_history");
pub const DepositHistoryInfo: Item<DepositInfo> = Item::new("deposit_history_info");
