use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use cw_storage_plus::{Map, Item};
use ariel::types::PositionDirection;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeRecord {
    pub ts: i64,
    pub record_id: u128,
    pub user: Addr,
    pub direction: PositionDirection,
    pub base_asset_amount: u128,
    pub quote_asset_amount: u128,
    pub mark_price_before: u128,
    pub mark_price_after: u128,
    pub fee: u128,
    pub referrer_reward: u128,
    pub referee_discount: u128,
    pub token_discount: u128,
    pub liquidation: bool,
    pub market_index: u64,
    pub oracle_price: i128,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeInfo {
    pub len: i64,
}

pub const TradeHistory: Map<(u64,Addr),  TradeRecord> = Map::new("trade_history");
pub const TradeHistoryInfo: Item<TradeInfo> = Item::new("trade_history_info");
