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
    pub positions: Addr,

    // upgrade-ability
    pub padding0: u128,
    pub padding1: u128,
    pub padding2: u128,
    pub padding3: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserPositions {
    pub user: Addr,
    pub positions: [MarketPosition; 5],
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

    // upgrade-ability
    pub padding0: u128,
    pub padding1: u128,
}

impl MarketPosition {
    pub fn is_for(&self, market_index: u64) -> bool {
        self.market_index == market_index && self.is_open_position()
    }

    pub fn is_open_position(&self) -> bool {
        self.base_asset_amount != 0
    }
}



pub const CONFIG: Map<UserPositions, u64> = Map::new("user_positions");
