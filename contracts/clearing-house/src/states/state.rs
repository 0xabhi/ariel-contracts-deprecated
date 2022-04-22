use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_controllers::Admin;
use cw_storage_plus::Item;

use ariel::types::{FeeStructure, OracleGuardRails};

use super::order::OrderState;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub exchange_paused: bool,
    pub funding_paused: bool,
    pub admin_controls_prices: bool,
    pub collateral_vault: Addr,
    pub insurance_vault: Addr,
    pub oracle: Addr,
    pub margin_ratio_initial: Uint128,
    pub margin_ratio_maintenance: Uint128,
    pub margin_ratio_partial: Uint128,
    
    pub partial_liquidation_close_percentage: Decimal,
    
    pub partial_liquidation_penalty_percentage: Decimal,
    
    pub full_liquidation_penalty_percentage: Decimal,

    
    pub partial_liquidation_liquidator_share_denominator: u64,
    pub full_liquidation_liquidator_share_denominator: u64,

    pub max_deposit: Uint128,
    pub markets_length: u64,
}

pub const STATE: Item<State> = Item::new("state");
pub const ADMIN: Admin = Admin::new("admin");
pub const FEESTRUCTURE: Item<FeeStructure> = Item::new("fee_structure");
pub const ORACLEGUARDRAILS: Item<OracleGuardRails> = Item::new("oracle_guard_rails");
pub const ORDERSTATE: Item<OrderState> = Item::new("order_state");