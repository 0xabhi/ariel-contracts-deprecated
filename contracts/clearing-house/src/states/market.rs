use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Map;
use cosmwasm_std::{Addr};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Market {
    pub market_name: String,
    pub initialized: bool,
    pub base_asset_amount_long: i128,
    pub base_asset_amount_short: i128,
    pub base_asset_amount: i128, // net market bias
    pub open_interest: u128,     // number of users in a position
    pub amm : Amm,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Amm {
    pub oracle: Addr,
    pub oracle_source: OracleSource,
    pub base_asset_reserve: u128,
    pub quote_asset_reserve: u128,
    pub cumulative_repeg_rebate_long: u128,
    pub cumulative_repeg_rebate_short: u128,
    pub cumulative_funding_rate_long: i128,
    pub cumulative_funding_rate_short: i128,
    pub last_funding_rate: i128,
    pub last_funding_rate_ts: i64,
    pub funding_period: i64,
    pub last_oracle_price_twap: i128,
    pub last_mark_price_twap: u128,
    pub last_mark_price_twap_ts: i64,
    pub sqrt_k: u128,
    pub peg_multiplier: u128,
    pub total_fee: u128,
    pub total_fee_minus_distributions: u128,
    pub total_fee_withdrawn: u128,
    pub minimum_trade_size: u128,
    pub last_oracle_price_twap_ts: i64,
    pub last_oracle_price: i128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum OracleSource {
    Oracle,
    Simulated,
    Zero,
}

impl Default for OracleSource {
    fn default() -> Self {
        OracleSource::Oracle
    }
}

pub const Markets: Map<u64, Market> = Map::new("markets");
