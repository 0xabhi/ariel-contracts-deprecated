use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Map, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentRecord {
    pub ts: u64,
    pub record_id: u64,
    pub user: Addr,
    pub market_index: u64,
    pub funding_payment: i128,
    pub base_asset_amount: i128,
    pub user_last_cumulative_funding: i128,
    pub user_last_funding_rate_ts: u64,
    pub amm_cumulative_funding_long: i128,
    pub amm_cumulative_funding_short: i128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentInfo {
    pub len: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingRateRecord {
    pub ts: u64,
    pub record_id: u64,
    pub market_index: u64,
    pub funding_rate: i128,
    pub cumulative_funding_rate_long: i128,
    pub cumulative_funding_rate_short: i128,
    pub oracle_price_twap: i128,
    pub mark_price_twap: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingRateInfo {
    pub len: u64,
}

pub const FUNDING_RATE_HISTORY: Map<u64,  FundingRateRecord> = Map::new("funding_payment_history");
pub const FUNDING_RATE_HISTORY_INFO: Item<FundingRateInfo> = Item::new("funding_payment_history_info");

pub const FUNDING_PAYMENT_HISTORY: Map<(&Addr, u64),  FundingPaymentRecord> = Map::new("funding_history");
pub const FUNDING_PAYMENT_HISTORY_INFO: Item<FundingPaymentInfo> = Item::new("funding_history_info");