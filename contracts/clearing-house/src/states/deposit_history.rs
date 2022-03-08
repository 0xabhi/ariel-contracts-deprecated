use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};

use std::vec::Vec;
use cw_storage_plus::Map;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositHistory {
    deposit_records:  Vec<DepositRecord>
}

impl Default for DepositHistory {
    fn default() -> Self {
        DepositHistory {
            deposit_records: Vec::new()
        }
    }
}

impl DepositHistory {
    pub fn append(&mut self, pos: DepositRecord) {
        self.deposit_records.push(pos)
    }

    pub fn length(&self) -> usize {
        self.deposit_records.len()
    }

    pub fn record_at_index(&self,  at_index : usize) -> DepositRecord {
        self.deposit_records[at_index]
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
