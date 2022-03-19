use cosmwasm_std::Addr;

use crate::error::ContractError;

use ariel::types::OracleGuardRails;

use crate::states::market::Amm;

use crate::helpers::amm;

pub fn block_operation(
    a: &Amm,
    oracle_account_info: &Addr,
    clock_slot: u64,
    guard_rails: &OracleGuardRails,
    precomputed_mark_price: Option<u128>,
) -> Result<(bool, i128), ContractError> {
    let oracle_is_valid =
        amm::is_oracle_valid(a, oracle_account_info, clock_slot, &guard_rails)?;
    let (oracle_price, _, oracle_mark_spread_pct) = amm::calculate_oracle_mark_spread_pct(
        &a,
        &oracle_account_info,
        0,
        clock_slot,
        precomputed_mark_price,
    )?;
    let is_oracle_mark_too_divergent =
        amm::is_oracle_mark_too_divergent(oracle_mark_spread_pct, &guard_rails)?;

    let block = !oracle_is_valid || is_oracle_mark_too_divergent;
    return Ok((block, oracle_price));
}
