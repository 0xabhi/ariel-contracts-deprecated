use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
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
    pub base_asset_amount_filled: u128,
    pub quote_asset_amount_filled: u128,
    pub fee: u128,
    pub filler_reward: u128,
    pub quote_asset_amount_surplus: u128,
    pub position_index: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderHisInfo {
    pub len: u64,
}

pub const OrderHistory: Map<u64,  OrderRecord> = Map::new("order_history");
pub const OrderHistoryInfo: Item<OrderHisInfo> = Item::new("order_history_info");