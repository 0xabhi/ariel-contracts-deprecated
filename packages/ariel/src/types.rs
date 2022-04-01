use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, JsonSchema, Copy, Serialize, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, JsonSchema, Copy, Serialize, Deserialize, PartialEq)]
pub enum SwapDirection {
    Add,
    Remove,
}

impl Default for SwapDirection {
    fn default() -> Self {
        SwapDirection::Add
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeStructure {
    pub fee_numerator: u128,
    pub fee_denominator: u128,

    pub first_tier: DiscountTokenTier,
    pub second_tier: DiscountTokenTier,
    pub third_tier: DiscountTokenTier,
    pub fourth_tier: DiscountTokenTier,

    pub referrer_reward_numerator: u128,
    pub referrer_reward_denominator: u128,
    pub referee_discount_numerator: u128,
    pub referee_discount_denominator: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DiscountTokenTier {
    pub minimum_balance: u64,
    pub discount_numerator: u128,
    pub discount_denominator: u128,
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

pub enum DepositDirection {
    DEPOSIT,
    WITHDRAW,
}

impl Default for DepositDirection {
    fn default() -> Self {
        DepositDirection::DEPOSIT
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum OracleSource {
    Oracle,
    Simulated,
    Zero
}


impl Default for OracleSource {
    // UpOnly
    fn default() -> Self {
        OracleSource::Oracle
    }
}
