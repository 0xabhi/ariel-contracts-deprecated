use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub clearing_house: Addr,
    pub total_deopsit: Uint128
}

pub const CONFIG: Item<Config> = Item::new("config");
