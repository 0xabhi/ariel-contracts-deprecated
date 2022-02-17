use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Decimal256, Uint256};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub leverage: Uint128,
    pub trade_paused: bool,
    pub deposit_paused: bool,
    pub price_controlled_by_admin: bool,
    pub collateral_fund: Addr,
    pub insurance_vault: Addr,
    pub initial_margin_ratio: Decimal256,
    pub maintenance_margin_ratio: Decimal256,
    pub liquidation_penalty: Decimal256,
    pub liquidator_reward: Decimal256,
    pub fee_percentage: Decimal256,
    pub max_deposit: Uint256,
}

pub struct Market {
    pub index: u16,
    pub v_amm: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
