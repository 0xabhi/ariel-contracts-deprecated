use cosmwasm_std::Addr;

use crate::error::ContractError;

use ariel::types::{DiscountTokenTier, FeeStructure};

pub fn calculate(
    quote_asset_amount: u128,
    fee_structure: &FeeStructure,
    discount_token_amt: u64,
    referrer: &Option<Addr>,
) -> Result<(u128, u128, u128, u128, u128), ContractError> {
    let fee = quote_asset_amount
        .checked_mul(fee_structure.fee_numerator)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(fee_structure.fee_denominator)
        .ok_or_else(|| (ContractError::MathError))?;

    let token_discount = calculate_token_discount(fee, fee_structure, discount_token_amt);

    let (referrer_reward, referee_discount) =
        calculate_referral_reward_and_referee_discount(fee, fee_structure, referrer)?;

    let user_fee = fee
        .checked_sub(token_discount)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_sub(referee_discount)
        .ok_or_else(|| (ContractError::MathError))?;

    let fee_to_market = user_fee
        .checked_sub(referrer_reward)
        .ok_or_else(|| (ContractError::MathError))?;

    return Ok((
        user_fee,
        fee_to_market,
        token_discount,
        referrer_reward,
        referee_discount,
    ));
}

fn calculate_token_discount(
    fee: u128,
    fee_structure: &FeeStructure,
    discount_token_amt: u64,
) -> u128 {
    if discount_token_amt == 0 {
        return 0;
    }

    if let Some(discount) =
        calculate_token_discount_for_tier(fee, &fee_structure.first_tier, discount_token_amt)
    {
        return discount;
    }

    if let Some(discount) =
        calculate_token_discount_for_tier(fee, &fee_structure.second_tier, discount_token_amt)
    {
        return discount;
    }

    if let Some(discount) =
        calculate_token_discount_for_tier(fee, &fee_structure.third_tier, discount_token_amt)
    {
        return discount;
    }

    if let Some(discount) =
        calculate_token_discount_for_tier(fee, &fee_structure.fourth_tier, discount_token_amt)
    {
        return discount;
    }

    return 0;
}

fn calculate_token_discount_for_tier(
    fee: u128,
    tier: &DiscountTokenTier,
    discount_token_amt: u64,
) -> Option<u128> {
    if discount_token_amt >= tier.minimum_balance {
        return Some(
            fee.checked_mul(tier.discount_numerator)?
                .checked_div(tier.discount_denominator)?,
        );
    }
    return None;
}

fn calculate_referral_reward_and_referee_discount(
    fee: u128,
    fee_structure: &FeeStructure,
    referrer: &Option<Addr>,
) -> Result<(u128, u128), ContractError> {
    if referrer.is_none() {
        return Ok((0, 0));
    }

    let referrer_reward = fee
        .checked_mul(fee_structure.referrer_reward_numerator)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(fee_structure.referrer_reward_denominator)
        .ok_or_else(|| (ContractError::MathError))?;

    let referee_discount = fee
        .checked_mul(fee_structure.referee_discount_numerator)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(fee_structure.referee_discount_denominator)
        .ok_or_else(|| (ContractError::MathError))?;

    return Ok((referrer_reward, referee_discount));
}
