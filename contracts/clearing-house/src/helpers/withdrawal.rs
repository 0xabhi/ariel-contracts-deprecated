use crate::error::ContractError;

/// Calculates how much of withdrawal must come from collateral vault and how much comes from insurance vault
pub fn calculate_withdrawal_amounts(
    amount: u64,
    balance_collateral: u64,
    balance_insurance: u64
) -> Result<(u64, u64), ContractError> {
    return Ok(
        if balance_collateral >= amount {
            (amount, 0)
        } else if balance_insurance > amount - balance_collateral
        {
            (balance_collateral, amount - balance_collateral)
        } else {
            (balance_collateral, balance_insurance)
        }
    );
}
