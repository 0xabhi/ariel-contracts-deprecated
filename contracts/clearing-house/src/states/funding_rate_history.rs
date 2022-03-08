use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::vec::Vec;
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingRateHistory {
    funding_rate_records: Vec<FundingRateRecord>,
}

impl Default for FundingRateHistory {
    fn default() -> Self {
        FundingRateHistory {
            funding_rate_records: Vec::new()
        }
    }
}

impl FundingRateHistory {
    pub fn append(&mut self, pos: FundingRateRecord) {
        self.funding_rate_records.push(pos)
    }

    pub fn length(&self) -> usize {
        self.funding_rate_records.len()
    }

    pub fn record_at_index(&self,  at_index : usize) -> FundingRateRecord {
        self.funding_rate_records[at_index]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingRateRecord {
    pub ts: i64,
    pub record_id: u128,
    pub market_index: u64,
    pub funding_rate: i128,
    pub cumulative_funding_rate_long: i128,
    pub cumulative_funding_rate_short: i128,
    pub oracle_price_twap: i128,
    pub mark_price_twap: u128,
}


pub const CONFIG: Map<FundingRateHistory, u64> = Map::new("funding_rate_history");