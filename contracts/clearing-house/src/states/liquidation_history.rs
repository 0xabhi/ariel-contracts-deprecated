use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Decimal256};
use cw_storage_plus::Map;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidationHistory {
    pub timestamp: u64,
    pub record_id: u64,
    pub user_address: Addr,
    pub funding_payment_amount: Uint128,
    pub base_asset_value: Uint128,
    pub base_asset_value_closed: Uint128,
    pub liquidation_fee: Uint128,
    pub liquidator_fee: Uint128,
    pub insurance_fund_fee: Uint128,
    pub liquidator: Addr,
    pub total_collateral: Uint128,
    pub collateral: Uint128,
    pub unrealized_pnl: Uint128,
    pub margin_ratio: Decimal256,
}

//TODO:: making the funding rate history map to composit key
pub const CONFIG: Map<LiquidationHistory, u64> = Map::new("funding_payment_history");
