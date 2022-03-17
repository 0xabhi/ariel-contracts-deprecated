use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{FeeStructure, OracleGuardRails, PositionDirection, SwapDirection};

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
        amm_periodicity: i64,
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
        user: Addr,
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
        fees: FeeStructure,
    },
    UpdateOraceGuardRails {
        oracle_guard_rails: OracleGuardRails,
    },
    // will move to admin controller
    UpdateAdmin {
        admin: Addr,
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
