use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeHistory {
    pub timestamp: u64,
    pub record_id: u64,
    pub user_address: Addr,
    pub market_id: u64,
    pub direction: Direction,
    pub funding_payment_amount: Uint128,
    pub base_asset_amount: Uint128,
    pub last_cumulative_funding: Uint128,
    pub last_funding_rate_timestamp: u64,
    pub long_cumulative_funding: Uint128,
    pub short_cumulative_funding: Uint128,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Direction{
    Long,
    Short
}

//TODO:: making the funding rate history map to composit key
pub const CONFIG: Map< &(Addr, u64), TradeHistory> = Map::new("funding_payment_history");
