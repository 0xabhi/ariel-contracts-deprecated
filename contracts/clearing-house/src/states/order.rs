use ariel::types::{Order};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

// use ariel::types::OracleSource;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderState {
    pub min_order_quote_asset_amount: Uint128, // minimum est. quote_asset_amount for place_order to succeed
    // pub reward_numerator: Uint128,
    // pub reward_denominator: Uint128,
    pub reward: Decimal,
    pub time_based_reward_lower_bound: Uint128, // minimum filler reward for time-based reward
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderInfo {
    pub len: u64,
}

pub const ORDERS: Map<((&Addr, u64), u64), Order> = Map::new("orders");
pub const ORDERS_INFO: Item<OrderInfo> = Item::new("order_info");

pub fn has_oracle_price_offset(oo: &Order) -> bool {
    oo.oracle_price_offset.i128() != 0
}

pub fn get_limit_price(
    oo: &Order,
    valid_oracle_price: Option<i128>,
) -> Result<Uint128, ContractError> {
    // the limit price can be hardcoded on order or derived from oracle_price + oracle_price_offset
    let price = if has_oracle_price_offset(oo) {
        if let Some(oracle_price) = valid_oracle_price {
            let limit_price = oracle_price
                .checked_add(oo.oracle_price_offset.i128())
                .ok_or_else(|| (ContractError::MathError))?;

            if limit_price <= 0 {
                return Err(ContractError::InvalidOracleOffset);
            }

            Uint128::from(limit_price.unsigned_abs())
        } else {
            return Err(ContractError::OracleNotFoundToOffset);
        }
    } else {
        oo.price
    };
    Ok(price)
}
