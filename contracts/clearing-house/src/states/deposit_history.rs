
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};

use std::vec::Vec;
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositHistory {
    head: u64,
    deposit_records:  Vec<DepositRecord>
}

impl Default for DepositHistory {
    fn default() -> Self {
        DepositHistory {
            head: 0,
            deposit_records: Vec::new()
        }
    }
}

impl DepositHistory {
    pub fn append(&mut self, pos: DepositRecord) {
        self.deposit_records[DepositHistory::index_of(self.head)] = pos;
        self.head = (self.head + 1) % 1024;
    }

    pub fn index_of(counter: u64) -> usize {
        std::convert::TryInto::try_into(counter).unwrap()
    }

    pub fn next_record_id(&self) -> u128 {
        let prev_trade_id = if self.head == 0 { 1023 } else { self.head - 1 };
        let prev_trade = &self.deposit_records[DepositHistory::index_of(prev_trade_id)];
        prev_trade.record_id + 1
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum DepositDirection {
    DEPOSIT,
    WITHDRAW,
}

impl Default for DepositDirection {
    // UpOnly
    fn default() -> Self {
        DepositDirection::DEPOSIT
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[derive(Default)]
pub struct DepositRecord {
    pub ts: i64,
    pub record_id: u128,
    pub user_authority: Addr,
    pub user: Addr,
    pub direction: DepositDirection,
    pub collateral_before: u128,
    pub cumulative_deposits_before: i128,
    pub amount: u64,
}

pub const CONFIG: Map<DepositHistory, u64> = Map::new("deposit_history");
