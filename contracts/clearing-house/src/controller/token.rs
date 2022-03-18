use std::fmt::Result;

use cosmwasm_std::Addr;
// use crate::error::ContractError;

pub fn send<'info>(
    from: &Addr,
    to: &Addr,
    amount: u64,
) -> Result {
    // return token::transfer(cpi_context, amount);
    Ok(())
}

pub fn receive<'info>(
    from: &Addr,
    to: &Addr,
    amount: u64,
) -> Result {
    // return token::transfer(cpi_context, amount);
    Ok(())
}
