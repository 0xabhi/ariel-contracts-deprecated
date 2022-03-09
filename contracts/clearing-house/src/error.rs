use cosmwasm_std::StdError;
use thiserror::Error;

pub type ClearingHouseResult<T = ()> = std::result::Result<T, ContractError>;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Unauthorized")]
    Unauthorized {},
    #[error("Clearing house not collateral account owner")]
    InvalidCollateralAccountAuthority,
    #[error("Clearing house not insurance account owner")]
    InvalidInsuranceAccountAuthority,
    #[error("Insufficient deposit")]
    InsufficientDeposit,
    #[error("Insufficient collateral")]
    InsufficientCollateral,
    #[error("Sufficient collateral")]
    SufficientCollateral,
    #[error("Max number of positions taken")]
    MaxNumberOfPositions,
    #[error("Admin Controls Prices Disabled")]
    AdminControlsPricesDisabled,
    #[error("Market Index Not Initialized")]
    MarketIndexNotInitialized,
    #[error("Market Index Already Initialized")]
    MarketIndexAlreadyInitialized,
    #[error("User Account And User Positions Account Mismatch")]
    UserAccountAndUserPositionsAccountMismatch,
    #[error("User Has No Position In Market")]
    UserHasNoPositionInMarket,
    #[error("Invalid Initial Peg")]
    InvalidInitialPeg,
    #[error("AMM repeg already configured with amt given")]
    InvalidRepegRedundant,
    #[error("AMM repeg incorrect repeg direction")]
    InvalidRepegDirection,
    #[error("AMM repeg out of bounds pnl")]
    InvalidRepegProfitability,
    #[error("Slippage Outside Limit Price")]
    SlippageOutsideLimit,
    #[error("Trade Size Too Small")]
    TradeSizeTooSmall,
    #[error("Price change too large when updating K")]
    InvalidUpdateK,
    #[error("Admin tried to withdraw amount larger than fees collected")]
    AdminWithdrawTooLarge,
    #[error("Math Error")]
    MathError,
    #[error("Conversion to u128/u64 failed with an overflow or underflow")]
    BnConversionError,
    #[error("Clock unavailable")]
    ClockUnavailable,
    #[error("Unable To Load Oracles")]
    UnableToLoadOracle,
    #[error("Oracle/Mark Spread Too Large")]
    OracleMarkSpreadLimit,
    #[error("Clearing House history already initialized")]
    HistoryAlreadyInitialized,
    #[error("Exchange is paused")]
    ExchangePaused,
    #[error("Invalid whitelist token")]
    InvalidWhitelistToken,
    #[error("Whitelist token not found")]
    WhitelistTokenNotFound,
    #[error("Invalid discount token")]
    InvalidDiscountToken,
    #[error("Discount token not found")]
    DiscountTokenNotFound,
    #[error("Invalid referrer")]
    InvalidReferrer,
    #[error("Referrer not found")]
    ReferrerNotFound,
    #[error("InvalidOracle")]
    InvalidOracle,
    #[error("OracleNotFound")]
    OracleNotFound,
    #[error("Liquidations Blocked By Oracle")]
    LiquidationsBlockedByOracle,
    #[error("Can not deposit more than max deposit")]
    UserMaxDeposit,
    #[error("Can not delete user that still has collateral")]
    CantDeleteUserWithCollateral,
    #[error("AMM funding out of bounds pnl")]
    InvalidFundingProfitability,
    #[error("Casting Failure")]
    CastingFailure,
}

#[macro_export]
macro_rules! wrap_error {
    ($err:expr) => {{
        || {
            msg!("Error thrown at {}:{}", file!(), line!());
            $err
        }
    }};
}

#[macro_export]
macro_rules! math_error {
    () => {{
        || {
            let error_code = ErrorCode::MathError;
            msg!("Error {} thrown at {}:{}", error_code, file!(), line!());
            error_code
        }
    }};
}

