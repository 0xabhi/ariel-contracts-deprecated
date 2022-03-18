use crate::error::ContractError;

use crate::states::trade_history::PositionDirection;
use crate::states::state::SwapDirection;
use crate::states::market::Amm;
use crate::states::user::Position;

use crate::helpers::amm;
use crate::helpers::amm::calculate_quote_asset_amount_swapped;
use crate::helpers::pnl::calculate_pnl;

pub fn calculate_base_asset_value_and_pnl(
    market_position: &Position,
    a: &Amm,
) -> Result<(u128, i128), ContractError> {
    return _calculate_base_asset_value_and_pnl(
        market_position.base_asset_amount,
        market_position.quote_asset_amount,
        a,
    );
}

pub fn _calculate_base_asset_value_and_pnl(
    base_asset_amount: i128,
    quote_asset_amount: u128,
    a: &Amm,
) -> Result<(u128, i128), ContractError> {
    if base_asset_amount == 0 {
        return Ok((0, 0));
    }

    let swap_direction = swap_direction_to_close_position(base_asset_amount);

    let (new_quote_asset_reserve, _new_base_asset_reserve) = 
        amm::calculate_swap_output(
        base_asset_amount.unsigned_abs(),
        a.base_asset_reserve,
        swap_direction,
        a.sqrt_k,
    )?;

    let base_asset_value = calculate_quote_asset_amount_swapped(
        a.quote_asset_reserve,
        new_quote_asset_reserve,
        swap_direction,
        a.peg_multiplier,
    )?;

    let pnl = calculate_pnl(base_asset_value, quote_asset_amount, swap_direction)?;

    return Ok((base_asset_value, pnl));
}

pub fn direction_to_close_position(base_asset_amount: i128) -> PositionDirection {
    if base_asset_amount > 0 {
        PositionDirection::Short
    } else {
        PositionDirection::Long
    }
}

pub fn swap_direction_to_close_position(base_asset_amount: i128) -> SwapDirection {
    if base_asset_amount >= 0 {
        SwapDirection::Add
    } else {
        SwapDirection::Remove
    }
}
