use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use cw_storage_plus::{Map, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingRateRecord {
    pub ts: i64,
    pub record_id: usize,
    pub market_index: u64,
    pub funding_rate: i128,
    pub cumulative_funding_rate_long: i128,
    pub cumulative_funding_rate_short: i128,
    pub oracle_price_twap: i128,
    pub mark_price_twap: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingRateInfo {
    pub len: i64,
}

pub const FundingRateHistory: Map<u64,  FundingRateRecord> = Map::new("funding_payment_history");
pub const FundingRateHistoryInfo: Item<FundingRateInfo> = Item::new("funding_payment_history_info");
