use cosmwasm_std::Uint128;

use crate::error::ContractError;

pub fn calculate_updated_collateral(collateral: Uint128, pnl: i128) -> Result<Uint128, ContractError> {
    return Ok(if pnl.is_negative() && pnl.unsigned_abs() > collateral.u128() {
        Uint128::zero()
    } else if pnl > 0 {
        collateral
            .checked_add(Uint128::from(pnl.unsigned_abs()))?
    } else {
        collateral
            .checked_sub(Uint128::from(pnl.unsigned_abs()))?
    });
}
