use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{PositionDirection};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
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
        amm_base_asset_reserve: u128,
        amm_quote_asset_reserve: u128,
        amm_periodicity: u128,
        amm_peg_multiplier: u128,
    },
    //deposit collateral, updates user struct
    DepositCollateral {
        amount: i128,
    },
    //user function withdraw collateral, updates user struct
    WithdrawCollateral {
        amount: i128,
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
        denominator: u128,
    },
    UpdateFullLiquidationLiquidatorShareDenominator {
        denominator: u128,
    },
    UpdateFee {
        fee_numerator: u128,
        fee_denominator: u128,
        t1_minimum_balance: u64,
        t1_discount_numerator: u128,
        t1_discount_denominator: u128,

        t2_minimum_balance: u64,
        t2_discount_numerator: u128,
        t2_discount_denominator: u128,

        t3_minimum_balance: u64,
        t3_discount_numerator: u128,
        t3_discount_denominator: u128,

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
