use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Uint128, Addr};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub clearing_house: Addr,
    pub admin : Addr,
    pub total_deposit: Uint128,
    pub denom_stable: String
}

pub const STATE: Item<State> = Item::new("state");
