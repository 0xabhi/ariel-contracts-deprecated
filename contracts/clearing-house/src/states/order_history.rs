use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Map, Item};

use ariel::types::Order;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum OrderAction {
    Place,
    Cancel,
    Fill,
    Expire,
}

impl Default for OrderAction {
    // UpOnly
    fn default() -> Self {
        OrderAction::Place
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderRecord {
    pub ts: u64,
    pub user: Addr,
    pub order: Order,
    pub action: OrderAction,
    pub filler: Addr,
    pub trade_record_id: u64,
    pub base_asset_amount_filled: Uint128,
    pub quote_asset_amount_filled: Uint128,
    pub fee: Uint128,
    pub filler_reward: Uint128,
    pub quote_asset_amount_surplus: Uint128,
    pub position_index: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderHisInfo {
    pub len: u64,
}

pub const ORDER_HISTORY: Map<u64,  OrderRecord> = Map::new("order_history");
pub const ORDER_HISTORY_INFO: Item<OrderHisInfo> = Item::new("order_history_info");