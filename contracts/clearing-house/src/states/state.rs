use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};

use cw_storage_plus::Map;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub admin: Addr,
    pub exchange_paused: bool,
    pub funding_paused: bool,
    pub admin_controls_prices: bool,
    pub collateral_mint: Addr,
    pub collateral_vault: Addr,
    pub collateral_vault_authority: Addr,
    pub collateral_vault_nonce: u8,
    pub deposit_history: Addr,
    pub trade_history: Addr,
    pub funding_payment_history: Addr,
    pub funding_rate_history: Addr,
    pub liquidation_history: Addr,
    pub curve_history: Addr,
    pub insurance_vault: Addr,
    pub insurance_vault_authority: Addr,
    pub insurance_vault_nonce: u8,
    pub markets: Addr,
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
    pub fee_structure: FeeStructure,
    pub whitelist_mint: Addr,
    pub discount_mint: Addr,
    pub oracle_guard_rails: OracleGuardRails,
    pub max_deposit: u128,
    pub extended_curve_history: Addr,

    // upgrade-ability
    pub padding0: u128,
    pub padding1: u128,
    pub padding2: u128,
    pub padding3: u128,
    pub padding4: u128,
    pub padding5: u128,
}


pub const CONFIG: Map<State, u64> = Map::new("state");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleGuardRails {
    pub price_divergence: PriceDivergenceGuardRails,
    pub validity: ValidityGuardRails,
    pub use_for_liquidations: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceDivergenceGuardRails {
    pub mark_oracle_divergence_numerator: u128,
    pub mark_oracle_divergence_denominator: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidityGuardRails {
    pub slots_before_stale: i64,
    pub confidence_interval_max_size: u128,
    pub too_volatile_ratio: i128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeStructure {
    pub fee_numerator: u128,
    pub fee_denominator: u128,
    pub discount_token_tiers: DiscountTokenTiers,
    pub referral_discount: ReferralDiscount,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DiscountTokenTiers {
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
pub struct ReferralDiscount {
    pub referrer_reward_numerator: u128,
    pub referrer_reward_denominator: u128,
    pub referee_discount_numerator: u128,
    pub referee_discount_denominator: u128,
}
