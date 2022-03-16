use crate::states::state::SwapDirection;

use crate::error::ContractError;

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
        // base asset value is round down due to integer math
        // subtract one from pnl so that users who are short dont get an extra +1 pnl from integer division
        SwapDirection::Remove => cast_to_i128(entry_value)?
            .checked_sub(cast(exit_value)?)
            .ok_or_else(|| (ContractError::MathError))?
            .checked_sub(1)
            .ok_or_else(|| (ContractError::MathError))?,
    })
}
