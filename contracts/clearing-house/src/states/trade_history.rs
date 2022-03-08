use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::vec::Vec;
use cw_storage_plus::Map;

use cosmwasm_std::{Addr};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeHistory {
    trade_records: Vec<TradeRecord>,
}

impl Default for TradeHistory {
    fn default() -> Self {
        TradeHistory {
            trade_records: Vec::new()
        }
    }
}

impl TradeHistory {
    pub fn append(&mut self, pos: TradeRecord) {
        self.trade_records.push(pos)
    }

    pub fn length(&self) -> usize {
        self.trade_records.len()
    }

    pub fn record_at_index(&self,  at_index : usize) -> TradeRecord {
        self.trade_records[at_index]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PositionDirection {
    Long,
    Short,
}

impl Default for PositionDirection {
    // UpOnly
    fn default() -> Self {
        PositionDirection::Long
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeRecord {
    pub ts: i64,
    pub record_id: u128,
    pub user_authority: Addr,
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


pub const CONFIG: Map<TradeHistory, u64> = Map::new("liquidation_history");
