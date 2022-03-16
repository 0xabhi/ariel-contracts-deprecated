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
    pub positions_length: u128 ,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketPosition {
    pub market_index: u64,
    pub base_asset_amount: i128,
    pub quote_asset_amount: u128,
    pub last_cumulative_funding_rate: i128,
    pub last_cumulative_repeg_rebate: u128,
    pub last_funding_rate_ts: i64,
    pub stop_loss_price: u128,
    pub stop_loss_amount: u128,
    pub stop_profit_price: u128,
    pub stop_profit_amount: u128,
    pub transfer_to: Addr,
}

pub const USER : Map<Addr, User> = Map::new("users");
pub const POSITIONS: Map<(Addr, u128),  MarketPosition> = Map::new("market_positions");

pub fn is_for(m :MarketPosition, market_index: u64) -> bool {
    m.market_index == market_index && is_open_position(m)
}

pub fn is_open_position(m :MarketPosition) -> bool {
    m.base_asset_amount != 0
}