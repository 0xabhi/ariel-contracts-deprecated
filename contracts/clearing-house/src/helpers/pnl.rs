use crate::error::ContractError;

use ariel::types::SwapDirection;

use cosmwasm_std::Uint128;

pub fn calculate_pnl(
    exit_value: Uint128,
    entry_value: Uint128,
    swap_direction_to_close: SwapDirection,
) -> Result<i128, ContractError> {
    let exit_value_i128 =  exit_value.u128() as i128;
    let entry_value_i128 = entry_value.u128() as i128;
    Ok(match swap_direction_to_close {
        SwapDirection::Add => exit_value_i128
            .checked_sub(entry_value_i128).ok_or_else(|| (ContractError::MathError {}))?,
        SwapDirection::Remove => entry_value_i128
            .checked_sub(exit_value_i128).ok_or_else(|| (ContractError::MathError {}))?,
    })
}