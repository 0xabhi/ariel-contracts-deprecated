use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{DiscountTokenTier, DepositDirection, PositionDirection, OracleSource};

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserResponse {
    pub collateral: u128,
    pub cumulative_deposits: i128,
    pub total_fee_paid: u128,
    pub total_token_discount: u128,
    pub total_referral_reward: u128,
    pub total_referee_discount: u128,
    pub positions_length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserPositionResponse {
    pub base_asset_amount: i128,
    pub quote_asset_amount: u128,
    pub last_cumulative_funding_rate: i128,
    pub last_cumulative_repeg_rebate: u128,
    pub last_funding_rate_ts: i64,
    pub stop_loss_price: u128,
    pub stop_loss_amount: u128,
    pub stop_profit_price: u128,
    pub stop_profit_amount: u128,
    pub transfer_to: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdminResponse {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsExchangePausedResponse {
    pub exchange_paused: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsFundingPausedResponse {
    pub funding_paused: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdminControlsPricesResponse {
    pub admin_controls_prices: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultsResponse {
    pub insurance_vault: String,
    pub collateral_vault: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarginRatioResponse {
    pub margin_ratio_initial: u128,
    pub margin_ratio_maintenance: u128,
    pub margin_ratio_partial: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PartialLiquidationClosePercentageResponse {
    pub numerator: u128,
    pub denominator: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PartialLiquidationPenaltyPercentageResponse {
    pub numerator: u128,
    pub denominator: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FullLiquidationPenaltyPercentageResponse {
    pub numerator: u128,
    pub denominator: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PartialLiquidatorSharePercentageResponse {
    pub denominator: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FullLiquidatorSharePercentageResponse {
    pub denominator: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MaxDepositLimitResponse {
    pub max_deposit: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeStructureResponse {
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
pub struct CurveHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurveHistoryResponse {
    pub ts: i64,
    pub record_id: u128,
    pub market_index: u64,
    pub peg_multiplier_before: u128,
    pub base_asset_reserve_before: u128,
    pub quote_asset_reserve_before: u128,
    pub sqrt_k_before: u128,
    pub peg_multiplier_after: u128,
    pub base_asset_reserve_after: u128,
    pub quote_asset_reserve_after: u128,
    pub sqrt_k_after: u128,
    pub base_asset_amount_long: u128,
    pub base_asset_amount_short: u128,
    pub base_asset_amount: i128,
    pub open_interest: u128,
    pub total_fee: u128,
    pub total_fee_minus_distributions: u128,
    pub adjustment_cost: i128,
    pub oracle_price: i128,
    pub trade_record: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositHistoryResponse {
    pub ts: i64,
    pub record_id: u128,
    pub user: String,
    pub direction: DepositDirection,
    pub collateral_before: u128,
    pub cumulative_deposits_before: i128,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentHistoryResponse {
    pub ts: i64,
    pub record_id: u64,
    pub user: String,
    pub market_index: u64,
    pub funding_payment: i128,
    pub base_asset_amount: i128,
    pub user_last_cumulative_funding: i128,
    pub user_last_funding_rate_ts: i64,
    pub amm_cumulative_funding_long: i128,
    pub amm_cumulative_funding_short: i128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingRateHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingRateHistoryResponse {
    pub ts: i64,
    pub record_id: u64,
    pub market_index: u64,
    pub funding_rate: i128,
    pub cumulative_funding_rate_long: i128,
    pub cumulative_funding_rate_short: i128,
    pub oracle_price_twap: i128,
    pub mark_price_twap: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidationHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidationHistoryResponse {
    pub ts: i64,
    pub record_id: u128,
    pub user: String,
    pub partial: bool,
    pub base_asset_value: u128,
    pub base_asset_value_closed: u128,
    pub liquidation_fee: u128,
    pub fee_to_liquidator: u64,
    pub fee_to_insurance_fund: u64,
    pub liquidator: String,
    pub total_collateral: u128,
    pub collateral: u128,
    pub unrealized_pnl: i128,
    pub margin_ratio: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeHistoryResponse {
    pub ts: i64,
    pub record_id: u128,
    pub user: String,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketInfoResponse {
    pub market_name: String,
    pub initialized: bool,
    pub base_asset_amount_long: i128,
    pub base_asset_amount_short: i128,
    pub base_asset_amount: i128, // net market bias
    pub open_interest: u128,  
    pub oracle: String,
    pub oracle_source: OracleSource,
    pub base_asset_reserve: u128,
    pub quote_asset_reserve: u128,
    pub cumulative_repeg_rebate_long: u128,
    pub cumulative_repeg_rebate_short: u128,
    pub cumulative_funding_rate_long: i128,
    pub cumulative_funding_rate_short: i128,
    pub last_funding_rate: i128,
    pub last_funding_rate_ts: i64,
    pub funding_period: i64,
    pub last_oracle_price_twap: i128,
    pub last_mark_price_twap: u128,
    pub last_mark_price_twap_ts: i64,
    pub sqrt_k: u128,
    pub peg_multiplier: u128,
    pub total_fee: u128,
    pub total_fee_minus_distributions: u128,
    pub total_fee_withdrawn: u128,
    pub minimum_trade_size: u128,
    pub last_oracle_price_twap_ts: i64,
    pub last_oracle_price: i128,
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct Response {
//     pub length: u64,
// }