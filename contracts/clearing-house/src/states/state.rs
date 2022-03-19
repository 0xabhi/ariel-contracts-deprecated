use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub admin: Addr,
    pub exchange_paused: bool,
    pub funding_paused: bool,
    pub admin_controls_prices: bool,
    pub collateral_vault: Addr,
    pub insurance_vault: Addr,
    pub margin_ratio_initial: u128,
    pub margin_ratio_maintenance: u128,
    pub margin_ratio_partial: u128,
    pub partial_liquidation_close_percentage_numerator: u128,
    pub partial_liquidation_close_percentage_denominator: u128,
    pub partial_liquidation_penalty_percentage_numerator: u128,
    pub partial_liquidation_penalty_percentage_denominator: u128,
    pub full_liquidation_penalty_percentage_numerator: u128,
    pub full_liquidation_penalty_percentage_denominator: u128,
    pub partial_liquidation_liquidator_share_denominator: u64,
    pub full_liquidation_liquidator_share_denominator: u64,
    pub max_deposit: u128,

    pub fee_structure: FeeStructure,
    pub oracle_guard_rails: OracleGuardRails,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleGuardRails {
    pub use_for_liquidations: bool,
    // oracle price divergence rails
    pub mark_oracle_divergence_numerator: u128,
    pub mark_oracle_divergence_denominator: u128,
    // validity guard rails
    pub slots_before_stale: i64,
    pub confidence_interval_max_size: u128,
    pub too_volatile_ratio: i128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeStructure {
    pub fee_numerator: u128,
    pub fee_denominator: u128,
    
    pub referrer_reward_numerator: u128,
    pub referrer_reward_denominator: u128,
    pub referee_discount_numerator: u128,
    pub referee_discount_denominator: u128,

    pub first_tier: DiscountTokenTier,
    pub second_tier: DiscountTokenTier,
    pub third_tier: DiscountTokenTier,
    pub fourth_tier: DiscountTokenTier,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DiscountTokenTier {
    pub minimum_balance: u64,
    pub discount_numerator: u128,
    pub discount_denominator: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum SwapDirection {
    Add,
    Remove,
}

impl Default for SwapDirection {
    // UpOnly
    fn default() -> Self {
        SwapDirection::Add
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

pub const STATE: Item<State> = Item::new("state");