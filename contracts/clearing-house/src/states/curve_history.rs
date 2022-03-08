use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Map;

use std::vec::Vec;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurveHistory {
    pub curve_records: Vec<CurveRecord>
}

impl Default for CurveHistory {
    fn default() -> Self {
        CurveHistory {
            curve_records: Vec::new()
        }
    }
}

impl CurveHistory {
    pub fn append(&mut self, pos: CurveRecord) {
        self.curve_records.push(pos)
    }

    pub fn length(&self) -> usize {
        self.curve_records.len()
    }

    pub fn record_at_index(&self,  at_index : usize) -> CurveRecord {
        self.curve_records[at_index]
    }
}


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
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExtendedCurveHistory {
    curve_records: Vec<ExtendedCurveRecord>
}

impl Default for ExtendedCurveHistory {
    fn default() -> Self {
        ExtendedCurveHistory {
            curve_records: Vec::new()        }
    }
}

impl ExtendedCurveHistory {
    pub fn append(&mut self, pos: ExtendedCurveRecord) {
        self.curve_records.push(pos)
    }

    pub fn length(&self) -> usize {
        self.curve_records.len()
    }

    pub fn record_at_index(&self,  at_index : usize) -> ExtendedCurveRecord {
        self.curve_records[at_index]
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExtendedCurveRecord {
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
    pub padding: [u128; 5],
}

pub const CONFIG: Map<CurveHistory, u64> = Map::new("curve_history");
pub const CONFIG2: Map<ExtendedCurveHistory, u64> = Map::new("extended_curve_history");
