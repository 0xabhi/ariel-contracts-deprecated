use cosmwasm_std::{Decimal256, Uint128, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{PositionDirection, SwapDirection};

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
        market_index: u64
    },
    // liquidation policy to be discussed
    Liquidate {
        user: Addr,
        market_index: u64
    },
    MoveAMMPrice {
        base_asset_reserve: u128,
        quote_asset_reserve:u128,
        market_index: u64
    },
    //user function
    WithdrawFees {
        market_index: u64,
        amount: u64
    },
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
    //admin function
    UpdateFeePercentage {
        new_fee: Decimal256,
    },
    //admin function
    UpdateMaxDeposit {
        max_deposit: Decimal256,
    },
    //admin function
    AddMarket {
        v_amm: String,
        long_base_asset_amount: Uint128,
        short_base_asset_amount: Uint128,
    },
}
