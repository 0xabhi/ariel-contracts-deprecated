use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal256, Uint128};
use cw_storage_plus::{Item, Map};



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
    pub max_deposit: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Market {
    pub index: u16,
    pub v_amm: Addr,
    pub initialized: bool,
    pub long_base_asset_amount: Uint128,
    pub short_base_asset_amount: Uint128,
    pub base_asset_amount: Uint128,
    pub open_positions: u64,
}


pub const MARKETS: Map<u64, Market> = Map::new("markets");
pub const CONFIG: Item<Config> = Item::new("config");
