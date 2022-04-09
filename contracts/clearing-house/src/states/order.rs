use ariel::types::{Order, OrderType, PositionDirection, OrderTriggerCondition};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

// use ariel::types::OracleSource;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderState {
    pub order_history: Addr,
    pub order_filler_reward_structure: OrderFillerRewardStructure,
    pub min_order_quote_asset_amount: u128, // minimum est. quote_asset_amount for place_order to succeed
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderFillerRewardStructure {
    pub reward_numerator: u128,
    pub reward_denominator: u128,
    pub time_based_reward_lower_bound: u128, // minimum filler reward for time-based reward
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderParams {
    pub order_type: OrderType,
    pub direction: PositionDirection,
    pub quote_asset_amount: u128,
    pub base_asset_amount: u128,
    pub price: u128,
    pub market_index: u64,
    pub reduce_only: bool,
    pub post_only: bool,
    pub immediate_or_cancel: bool,
    pub trigger_price: u128,
    pub trigger_condition: OrderTriggerCondition,
    pub position_limit: u128,
    pub oracle_price_offset: i128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderInfo {
    pub len: i64,
}

pub const Orders: Map<(&Addr, u64), Order> = Map::new("orders");
pub const OrdersInfo: Item<OrderInfo> = Item::new("order_info");

pub fn has_oracle_price_offset(oo: &Order) -> bool {
    oo.oracle_price_offset != 0
}

pub fn get_limit_price(oo: &Order, valid_oracle_price: Option<i128>) -> Result<u128, ContractError> {
    // the limit price can be hardcoded on order or derived from oracle_price + oracle_price_offset
    let price = if has_oracle_price_offset(oo) {
        if let Some(oracle_price) = valid_oracle_price {
            let limit_price = oracle_price
                .checked_add(oo.oracle_price_offset)
                .ok_or_else(|| (ContractError::MathError))?;

            if limit_price <= 0 {
                return Err(ContractError::InvalidOracleOffset);
            }

            limit_price.unsigned_abs()
        } else {
            return Err(ContractError::OracleNotFoundToOffset);
        }
    } else {
        oo.price
    };
    Ok(price)
}
