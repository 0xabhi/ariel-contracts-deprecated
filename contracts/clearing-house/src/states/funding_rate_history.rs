use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Uint128;
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingRateHistory {
    pub timestamp: u64,
    pub record_id: u64,
    pub market_id: u64,
    pub funding_rate: Uint128,
    pub twap_mark_price: Uint128,
    pub twap_oracle_price: Uint128,
    pub long_cumulative_funding: Uint128,
    pub short_cumulative_funding: Uint128,
}

//TODO:: making the funding rate history map to composit key
pub const CONFIG: Map<FundingRateHistory, u64> = Map::new("funding_rate_history");
