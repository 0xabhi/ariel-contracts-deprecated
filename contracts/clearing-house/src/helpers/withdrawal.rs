use cosmwasm_std::Uint128;

use crate::error::ContractError;

/// Calculates how much of withdrawal must come from collateral vault and how much comes from insurance vault
pub fn calculate_withdrawal_amounts(
    amount: Uint128,
    balance_collateral: Uint128,
    balance_insurance: Uint128
) -> Result<(Uint128, Uint128), ContractError> {
    return Ok(
        if balance_collateral.u128() >= amount.u128() {
            (amount, Uint128::zero())
        } else if balance_insurance.u128() > amount.u128() - balance_collateral.u128()
        {
            (balance_collateral, amount.checked_sub(balance_collateral)?)
        } else {
            (balance_collateral, balance_insurance)
        }
    );
}
