use cosmwasm_std::{Uint128, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{types::{DepositDirection, OracleSource, PositionDirection}, number::Number128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserResponse {
    pub collateral: Uint128,
    pub cumulative_deposits: Uint128,
    pub total_fee_paid: Uint128,
    pub total_token_discount: Uint128,
    pub total_referral_reward: Uint128,
    pub total_referee_discount: Uint128,
    pub positions_length: u64,
    pub referrer: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserPositionResponse {
    pub market_index: u64,
    pub base_asset_amount: Number128,
    pub quote_asset_amount: Uint128,
    pub last_cumulative_funding_rate: Number128,
    pub last_cumulative_repeg_rebate: Uint128,
    pub last_funding_rate_ts: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PositionResponse {
    pub market_index: u64,
    pub direction: PositionDirection,
    pub initial_size: Uint128,
    pub entry_notional: Number128,
    pub entry_price: Uint128,
    pub pnl: Number128
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
pub struct OracleResponse {
    pub oracle: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarginRatioResponse {
    pub margin_ratio_initial: Uint128,
    pub margin_ratio_maintenance: Uint128,
    pub margin_ratio_partial: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PartialLiquidationClosePercentageResponse {
    pub value: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PartialLiquidationPenaltyPercentageResponse {
    pub value: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FullLiquidationPenaltyPercentageResponse {
    pub value: Decimal,
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
    pub max_deposit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketLengthResponse {
    pub length: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeStructureResponse {
    pub fee: Decimal,
    pub first_tier_minimum_balance: Uint128,
    pub first_tier_discount : Decimal,
    pub second_tier_minimum_balance : Uint128,
    pub second_tier_discount : Decimal,
    pub third_tier_minimum_balance : Uint128,
    pub third_tier_discount : Decimal,
    pub fourth_tier_minimum_balance : Uint128,
    pub fourth_tier_discount : Decimal,
    pub referrer_reward : Decimal,
    pub referee_discount : Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleGuardRailsResponse {
    pub use_for_liquidations: bool,
    // oracle price divergence rails
    pub mark_oracle_divergence: Decimal,
    // validity guard rails
    pub slots_before_stale: Number128,
    pub confidence_interval_max_size: Uint128,
    pub too_volatile_ratio: Number128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderStateResponse {
    pub min_order_quote_asset_amount: Uint128, // minimum est. quote_asset_amount for place_order to succeed
    pub reward: Decimal,
    pub time_based_reward_lower_bound: Uint128, // minimum filler reward for time-based reward
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurveHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurveHistoryResponse {
    pub ts: u64,
    pub record_id: u64,
    pub market_index: u64,
    pub peg_multiplier_before: Uint128,
    pub base_asset_reserve_before: Uint128,
    pub quote_asset_reserve_before: Uint128,
    pub sqrt_k_before: Uint128,
    pub peg_multiplier_after: Uint128,
    pub base_asset_reserve_after: Uint128,
    pub quote_asset_reserve_after: Uint128,
    pub sqrt_k_after: Uint128,
    pub base_asset_amount_long: Uint128,
    pub base_asset_amount_short: Uint128,
    pub base_asset_amount: Number128,
    pub open_interest: Uint128,
    pub total_fee: Uint128,
    pub total_fee_minus_distributions: Uint128,
    pub adjustment_cost: Number128,
    pub oracle_price: Number128,
    pub trade_record: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositHistoryResponse {
    pub ts: u64,
    pub record_id: u64,
    pub user: String,
    pub direction: DepositDirection,
    pub collateral_before: Uint128,
    pub cumulative_deposits_before: Uint128,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentHistoryResponse {
    pub ts: u64,
    pub record_id: u64,
    pub user: String,
    pub market_index: u64,
    pub funding_payment: Number128,
    pub base_asset_amount: Number128,
    pub user_last_cumulative_funding: Number128,
    pub user_last_funding_rate_ts: u64,
    pub amm_cumulative_funding_long: Number128,
    pub amm_cumulative_funding_short: Number128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingRateHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingRateHistoryResponse {
    pub ts: u64,
    pub record_id: u64,
    pub market_index: u64,
    pub funding_rate: Number128,
    pub cumulative_funding_rate_long: Number128,
    pub cumulative_funding_rate_short: Number128,
    pub oracle_price_twap: Number128,
    pub mark_price_twap: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidationHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidationHistoryResponse {
    pub ts: u64,
    pub record_id: u64,
    pub user: String,
    pub partial: bool,
    pub base_asset_value: Uint128,
    pub base_asset_value_closed: Uint128,
    pub liquidation_fee: Uint128,
    pub fee_to_liquidator: u64,
    pub fee_to_insurance_fund: u64,
    pub liquidator: String,
    pub total_collateral: Uint128,
    pub collateral: Uint128,
    pub unrealized_pnl: Number128,
    pub margin_ratio: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeHistoryLengthResponse {
    pub length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeHistoryResponse {
    pub ts: u64,
    pub user: String,
    pub direction: PositionDirection,
    pub base_asset_amount: Uint128,
    pub quote_asset_amount: Uint128,
    pub mark_price_before: Uint128,
    pub mark_price_after: Uint128,
    pub fee: Uint128,
    pub referrer_reward: Uint128,
    pub referee_discount: Uint128,
    pub token_discount: Uint128,
    pub liquidation: bool,
    pub market_index: u64,
    pub oracle_price: Number128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketInfoResponse {
    pub market_name: String,
    pub initialized: bool,
    pub base_asset_amount_long: Number128,
    pub base_asset_amount_short: Number128,
    pub base_asset_amount: Number128, // net market bias
    pub open_interest: Uint128,
    pub oracle: String,
    pub oracle_source: OracleSource,
    pub base_asset_reserve: Uint128,
    pub quote_asset_reserve: Uint128,
    pub cumulative_repeg_rebate_long: Uint128,
    pub cumulative_repeg_rebate_short: Uint128,
    pub cumulative_funding_rate_long: Number128,
    pub cumulative_funding_rate_short: Number128,
    pub last_funding_rate: Number128,
    pub last_funding_rate_ts: u64,
    pub funding_period: u64,
    pub last_oracle_price_twap: Number128,
    pub last_mark_price_twap: Uint128,
    pub last_mark_price_twap_ts: u64,
    pub sqrt_k: Uint128,
    pub peg_multiplier: Uint128,
    pub total_fee: Uint128,
    pub total_fee_minus_distributions: Uint128,
    pub total_fee_withdrawn: Uint128,
    pub minimum_trade_size: Uint128,
    pub last_oracle_price_twap_ts: u64,
    pub last_oracle_price: Number128,
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct Response {
//     pub length: u64,
// }
