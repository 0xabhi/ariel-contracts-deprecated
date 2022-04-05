use crate::error::ContractError;

use ariel::types::SwapDirection;

use crate::helpers::casting::{cast, cast_to_i128};

pub fn calculate_pnl(
    exit_value: u128,
    entry_value: u128,
    swap_direction_to_close: SwapDirection,
) -> Result<i128, ContractError> {
    Ok(match swap_direction_to_close {
        SwapDirection::Add => cast_to_i128(exit_value)?
            .checked_sub(cast(entry_value)?)
            .ok_or_else(|| (ContractError::MathError))?,
        SwapDirection::Remove => cast_to_i128(entry_value)?
            .checked_sub(cast(exit_value)?)
            .ok_or_else(|| (ContractError::MathError))?,
    })
}
