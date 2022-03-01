use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128};

use crate::ContractError;

pub fn try_add_market(
    deps: DepsMut,
    info: MessageInfo,
    v_amm: String,
    long_base_asset_amount: Uint128,
    short_base_asset_amount: Uint128,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}
