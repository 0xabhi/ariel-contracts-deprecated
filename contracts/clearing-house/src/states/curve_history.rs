use ariel::number::Number128;
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Map, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Type {
    Repeg,
    UpdateK,
}

impl Default for Type {
    // UpOnly
    fn default() -> Self {
        Type::Repeg
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurveRecord {
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
pub struct CurveInfo {
    pub len: u64,
}

pub const CURVEHISTORY: Map<u64,  CurveRecord> = Map::new("curve_history");
pub const CURVE_HISTORY_INFO: Item<CurveInfo> = Item::new("curve_history_info");