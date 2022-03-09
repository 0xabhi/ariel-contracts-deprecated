use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::vec::Vec;
use cw_storage_plus::Map;

use cosmwasm_std::{Addr};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidationHistory {
    liquidation_records: Vec<LiquidationRecord>,
}

impl Default for LiquidationHistory {
    fn default() -> Self {
        LiquidationHistory {
            liquidation_records: Vec::new()
        }
    }
}

impl LiquidationHistory {
    pub fn append(&mut self, pos: LiquidationRecord) {
        self.liquidation_records.push(pos)
    }

    pub fn length(&self) -> usize {
        self.liquidation_records.len()
    }

    pub fn record_at_index(&self,  at_index : usize) -> LiquidationRecord {
        self.liquidation_records[at_index]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidationRecord {
    pub ts: i64,
    pub record_id: u128,
    pub user: Addr,
    pub partial: bool,
    pub base_asset_value: u128,
    pub base_asset_value_closed: u128,
    pub liquidation_fee: u128,
    pub fee_to_liquidator: u64,
    pub fee_to_insurance_fund: u64,
    pub liquidator: Addr,
    pub total_collateral: u128,
    pub collateral: u128,
    pub unrealized_pnl: i128,
    pub margin_ratio: u128,
}


pub const CONFIG: Map<LiquidationHistory, u64> = Map::new("liquidation_history");
