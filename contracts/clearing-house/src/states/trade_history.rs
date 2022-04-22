use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Map, Item};

use ariel::{types::PositionDirection, number::Number128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeRecord {
    pub ts: u64,
    pub user: Addr,
    pub direction: PositionDirection,
    pub base_asset_amount: Uint128,
    pub quote_asset_amount: Uint128,
    pub mark_price_before: Uint128,
    pub mark_price_after: Uint128,
    pub fee: Uint128,
    pub referrer_reward: Uint128,
    pub referee_discount: Uint128,
    pub token_discount: Uint128,
    pub liquidation: bool,
    pub market_index: u64,
    pub oracle_price: Number128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeInfo {
    pub len: u64,
}

pub const TRADE_HISTORY: Map<u64,  TradeRecord> = Map::new("trade_history");
pub const TRADE_HISTORY_INFO: Item<TradeInfo> = Item::new("trade_history_info");
