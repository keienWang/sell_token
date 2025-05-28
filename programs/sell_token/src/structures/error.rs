use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The PDA account does not match.")]
    PdaAccountIsNotMatch,

    #[msg("Sale amount is too low.")]
    SaleAmountTooLow,

    #[msg("Insufficient balance.")]
    InsufficientBalance,

    #[msg("Sale not active.")]
    SaleNotActive,

    #[msg("Sale ended.")]
    SaleEnded,

    #[msg("Insufficient tokens.")]
    InsufficientTokens,

    #[msg("Calculation error.")]
    CalculationError,

    #[msg("Sale not ended.")]
    SaleNotEnded,

    #[msg("No tokens to withdraw.")]
    NoTokensToWithdraw,

    #[msg("Unauthorized.")]
    Unauthorized,

    #[msg("Invalid price.")]
    InvalidPrice,

    #[msg("Invalid end time.")]
    InvalidEndTime,

    #[msg("Sale amount is too high.")]
    SaleAmountTooHigh,

    #[msg("Amount too small.")]
    AmountTooSmall,

    #[msg("Overflow.")]
    Overflow,
    
    #[msg("No tokens left for sale.")]
    NoTokensLeft,

    #[msg("Token balance mismatch.")]
    BalanceMismatch,

    #[msg("Token mint mismatch.")]
    TokenMintMismatch,

    #[msg("Token account mismatch.")]
    TokenAccountMismatch,

    #[msg("User already purchased.")]
    UserAlreadyPurchased,

    #[msg("User not purchased.")]
    UserNotPurchased,

    #[msg("MissingRequiredSignature.")]
    MissingRequiredSignature,
    
}

