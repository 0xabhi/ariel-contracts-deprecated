use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};

use std::vec::Vec;
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentHistory {
    funding_payment_records: Vec<FundingPaymentRecord>,
}

impl Default for FundingPaymentHistory {
    fn default() -> Self {
        FundingPaymentHistory {
            funding_payment_records: Vec::new()
        }
    }
}

impl FundingPaymentHistory {
    pub fn append(&mut self, pos: FundingPaymentRecord) {
        self.funding_payment_records.push(pos)
    }

    pub fn length(&self) -> usize {
        self.funding_payment_records.len()
    }

    pub fn record_at_index(&self,  at_index : usize) -> FundingPaymentRecord {
        self.funding_payment_records[at_index]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentRecord {
    pub ts: i64,
    pub record_id: usize,
    pub user: Addr,
    pub market_index: u64,
    pub funding_payment: i128,
    pub base_asset_amount: i128,
    pub user_last_cumulative_funding: i128,
    pub user_last_funding_rate_ts: i64,
    pub amm_cumulative_funding_long: i128,
    pub amm_cumulative_funding_short: i128,
}


pub const CONFIG: Map<FundingPaymentHistory, u64> = Map::new("funding_payment_history");
