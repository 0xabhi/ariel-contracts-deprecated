use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{DiscountTokenTier, OracleSource, Order, PositionDirection};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub collateral_vault: String,
    pub insurance_vault: String,
    pub admin_controls_prices: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // market initializer updates AMM structure
    InitializeMarket {
        market_index: u64,
        market_name: String,
        amm_base_asset_reserve: u128,
        amm_quote_asset_reserve: u128,
        amm_periodicity: i64,
        amm_peg_multiplier: u128,
        oracle_source: OracleSource,
        margin_ratio_initial: u32,
        margin_ratio_partial: u32,
        margin_ratio_maintenance: u32,
    },
    //deposit collateral, updates user struct
    DepositCollateral {
        amount: u64,
    },
    //user function withdraw collateral, updates user struct
    WithdrawCollateral {
        amount: u64,
    },
    OpenPosition {
        direction: PositionDirection,
        quote_asset_amount: u128,
        market_index: u64,
        limit_price: u128,
    },
    ClosePosition {
        market_index: u64,
    },

    // order related messages
    PlaceOrder {
        order: Order,
    },
    CancelOrder {
        order_id: u128,
    },
    CancelOrderByUserId {
        user_order_id: u8,
    },
    ExpireOrders {},
    FillOrder {
        order_id: u128,
    },
    PlaceAndFillOrder {
        order: Order,
    },

    // liquidation policy to be discussed
    Liquidate {
        user: String,
        market_index: u64,
    },
    MoveAMMPrice {
        base_asset_reserve: u128,
        quote_asset_reserve: u128,
        market_index: u64,
    },
    //user function
    WithdrawFees {
        market_index: u64,
        amount: u64,
    },

    // withdraw from insurance vault sends token but no logic

    //admin function
    WithdrawFromInsuranceVaultToMarket {
        market_index: u64,
        amount: u64,
    },
    //admin function
    RepegAMMCurve {
        new_peg_candidate: u128,
        market_index: u64,
    },

    UpdateAMMOracleTwap {
        market_index: u64,
    },

    ResetAMMOracleTwap {
        market_index: u64,
    },
    //user calls it we get the user identification from msg address sender
    SettleFundingPayment {},
    UpdateFundingRate {
        market_index: u64,
    },
    UpdateK {
        market_index: u64,
        sqrt_k: u128,
    },
    UpdateMarketMinimumTradeSize {
        market_index: u64,
        minimum_trade_size: u128,
    },
    UpdateMarginRatio {
        margin_ratio_initial: u128,
        margin_ratio_partial: u128,
        margin_ratio_maintenance: u128,
    },
    UpdatePartialLiquidationClosePercentage {
        numerator: u128,
        denominator: u128,
    },
    UpdatePartialLiquidationPenaltyPercentage {
        numerator: u128,
        denominator: u128,
    },
    UpdateFullLiquidationPenaltyPercentage {
        numerator: u128,
        denominator: u128,
    },
    UpdatePartialLiquidationLiquidatorShareDenominator {
        denominator: u64,
    },
    UpdateFullLiquidationLiquidatorShareDenominator {
        denominator: u64,
    },
    UpdateFee {
        fee_numerator: u128,
        fee_denominator: u128,
        first_tier: DiscountTokenTier,
        second_tier: DiscountTokenTier,
        third_tier: DiscountTokenTier,
        fourth_tier: DiscountTokenTier,
        referrer_reward_numerator: u128,
        referrer_reward_denominator: u128,
        referee_discount_numerator: u128,
        referee_discount_denominator: u128,
    },
    UpdateOraceGuardRails {
        use_for_liquidations: bool,
        mark_oracle_divergence_numerator: u128,
        mark_oracle_divergence_denominator: u128,
        slots_before_stale: i64,
        confidence_interval_max_size: u128,
        too_volatile_ratio: i128,
    },

    UpdateOrderFillerRewardSystem {
        reward_numerator: u128,
        reward_denominator: u128,
        time_based_reward_lower_bound: u128,
    },
    UpdateMarketOracle {
        market_index: u64,
        oracle: String,
        oracle_source: OracleSource,
    },

    UpdateMarketMinimumQuoteAssetTradeSize {
        market_index: u64,
        minimum_trade_size: u128,
    },

    UpdateMarketMinimumBaseAssetTradeSize {
        market_index: u64,
        minimum_trade_size: u128,
    },
    // will move to admin controller
    UpdateAdmin {
        admin: String,
    },
    UpdateMaxDeposit {
        max_deposit: u128,
    },
    UpdateExchangePaused {
        exchange_paused: bool,
    },
    DisableAdminControlsPrices {},
    UpdateFundingPaused {
        funding_paused: bool,
    },
}
