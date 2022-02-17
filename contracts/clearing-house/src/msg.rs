use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{ Uint128, Decimal256, Uint256 };



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg{
    pub leverage: Uint128,
    pub collateral_fund: String,
    pub insurance_vault: String,
    pub initial_margin_ratio: Decimal256,
    pub maintenance_margin_ratio: Decimal256,
    pub liquidation_penalty: Decimal256,
    pub liquidator_reward: Decimal256,
    pub fee_percentage: Decimal256,
    pub max_deposit: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    DepositCollateral { amount: Uint128 },
    WithdrawCollateral { amount: Uint128 },
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
}