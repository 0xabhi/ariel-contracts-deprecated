use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentHistory {
    head: u64,
    funding_payment_records: [FundingPaymentRecord; 1024],
}

impl Default for FundingPaymentHistory {
    fn default() -> Self {
        FundingPaymentHistory {
            head: 0,
            funding_payment_records: [FundingPaymentRecord::default(); 1024],
        }
    }
}

impl FundingPaymentHistory {
    pub fn append(&mut self, pos: FundingPaymentRecord) {
        self.funding_payment_records[FundingPaymentHistory::index_of(self.head)] = pos;
        self.head = (self.head + 1) % 1024;
    }

    pub fn index_of(counter: u64) -> usize {
        std::convert::TryInto::try_into(counter).unwrap()
    }

    pub fn next_record_id(&self) -> u128 {
        let prev_record_id = if self.head == 0 { 1023 } else { self.head - 1 };
        let prev_record =
            &self.funding_payment_records[FundingPaymentHistory::index_of(prev_record_id)];
        prev_record.record_id + 1
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentRecord {
    pub timestamp: u64,
    pub record_id: u64,
    pub user_address: Addr,
    pub market_id: u64,
    pub funding_payment_amount: Uint128,
    pub base_asset_amount: Uint128,
    pub last_cumulative_funding: Uint128,
    pub last_funding_rate_timestamp: u64,
    pub long_cumulative_funding: Uint128,
    pub short_cumulative_funding: Uint128,
}

//TODO:: making the funding rate history map to composit key
pub const CONFIG: Map<FundingPaymentHistory, u64> = Map::new("funding_payment_history");
