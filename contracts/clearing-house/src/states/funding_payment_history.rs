use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use cw_storage_plus::{Map, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentRecord {
    pub ts: i64,
    pub record_id: usize,
    pub user: Addr,
    pub market_index: u64,
    pub funding_payment: i128,
    pub base_asset_amount: i128,
    pub user_last_cumulative_funding: i128,
    pub user_last_funding_rate_ts: i64,
    pub amm_cumulative_funding_long: i128,
    pub amm_cumulative_funding_short: i128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingInfo {
    pub len: i64,
}

pub const FundingHistory: Map<(u64,Addr),  FundingPaymentRecord> = Map::new("funding_history");
pub const FundingHistoryInfo: Item<FundingInfo> = Item::new("funding_history_info");
