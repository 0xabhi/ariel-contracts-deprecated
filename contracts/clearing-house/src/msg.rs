use cosmwasm_std::{Decimal256, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub leverage: Uint128,
    pub collateral_fund: String,
    pub insurance_vault: String,
    pub initial_margin_ratio: Decimal256,
    pub maintenance_margin_ratio: Decimal256,
    pub liquidation_penalty: Decimal256,
    pub liquidator_reward: Decimal256,
    pub fee_percentage: Decimal256,
    pub max_deposit: Uint128,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // updates admin
    // UpdateAdmin {
    //     admin: String
    // },
    //user function
    DepositCollateral {
        amount: Uint128,
    },
    //user function
    WithdrawCollateral {
        amount: Uint128,
    },
    //admin function
    UpdateCollateralVault {
        vault: String,
    },
    //admin function
    UpdateInsuranceVault {
        vault: String,
    },
    //admin function
    UpdateMarginRatio {
        initial_mr: Decimal256,
        maintenance_mr: Decimal256,
    },
    //admin function
    UpdateLiquidationConfig{
        liquidation_penalty: Decimal256,
        liquidation_reward: Decimal256,
    },
    //admin function
    UpdateFeePercentage{
        new_fee: Decimal256,
    },
    //admin function
    UpdateMaxDeposit {
        max_deposit: Decimal256
    },
    //admin function
    AddMarket {
        v_amm: String,
        long_base_asset_amount: Uint128,
        short_base_asset_amount: Uint128,
    },
    //user function
    OpenPosition {
        market_index: u64
    },
    //user function
    ClosePosition {
        market_index: u64
    },

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetUser { user_address: String },
    GetConfig {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserResponse {
    pub user_address: String,
    pub free_collateral: Uint128,
    pub total_deposits: Uint128,
    pub total_paid_fees: Uint128,
}

// get config data response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub leverage: Uint128,
    pub collateral_fund: String,
    pub insurance_vault: String,
    pub initial_margin_ratio: Decimal256,
    pub maintenance_margin_ratio: Decimal256,
    pub liquidation_penalty: Decimal256,
    pub liquidator_reward: Decimal256,
    pub fee_percentage: Decimal256,
    pub max_deposit: Uint128,
}
