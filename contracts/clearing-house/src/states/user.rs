use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct User {
    pub user_address: Addr,
    pub free_collateral: Uint128,
    pub total_deposits: Uint128,
    pub total_paid_fees: Uint128,
}

pub const USER: Map<String, User> = Map::new("user");
