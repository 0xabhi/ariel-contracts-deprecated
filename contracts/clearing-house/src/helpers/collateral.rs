use crate::error::ContractError;

pub fn calculate_updated_collateral(collateral: u128, pnl: i128) -> Result<u128, ContractError> {
    return Ok(if pnl.is_negative() && pnl.unsigned_abs() > collateral {
        0
    } else if pnl > 0 {
        collateral
            .checked_add(pnl.unsigned_abs())
            .ok_or_else(|| (ContractError::MathError))?
    } else {
        collateral
            .checked_sub(pnl.unsigned_abs())
            .ok_or_else(|| (ContractError::MathError))?
    });
}
