use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use cw_storage_plus::{Map, Item};

use crate::states::order::Order;

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
    pub ts: i64,
    pub record_id: u128,
    pub user: Addr,
    pub order: Order,
    pub action: OrderAction,
    pub filler: Addr,
    pub trade_record_id: u128,
    pub base_asset_amount_filled: u128,
    pub quote_asset_amount_filled: u128,
    pub fee: u128,
    pub filler_reward: u128,
    pub quote_asset_amount_surplus: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderInfo {
    pub len: i64,
}

pub const OrderHistory: Map<(u64,Addr),  OrderRecord> = Map::new("order_history");
pub const OrderHistoryInfo: Item<OrderInfo> = Item::new("order_history_info");