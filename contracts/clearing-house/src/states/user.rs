use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Map;
use cosmwasm_std::{Addr};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct User {
    pub collateral: u128,
    pub cumulative_deposits: i128,
    pub total_fee_paid: u128,
    pub total_token_discount: u128,
    pub total_referral_reward: u128,
    pub total_referee_discount: u128,
    pub positions_length: u64,
    pub order_length: u64,
    pub referrer: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Position {
    pub market_index: u64,
    pub base_asset_amount: i128,
    pub quote_asset_amount: u128,
    pub last_cumulative_funding_rate: i128,
    pub last_cumulative_repeg_rebate: u128,
    pub last_funding_rate_ts: i64,
    pub open_orders: u128,
}

pub const Users : Map<&Addr, User> = Map::new("users");
pub const Positions: Map<(&Addr, u64),  Position> = Map::new("market_positions");

impl Position {
    pub fn is_for(&self, market_index: u64) -> bool {
        self.market_index == market_index && (self.is_open_position() || self.has_open_order())
    }

    pub fn is_available(&self) -> bool {
        !self.is_open_position() && !self.has_open_order()
    }

    pub fn is_open_position(&self) -> bool {
        self.base_asset_amount != 0
    }

    pub fn has_open_order(&self) -> bool {
        self.open_orders != 0
    }
}
