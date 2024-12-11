use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("At least one outcome is required")]
    NoOutcomes,
    #[msg("Liquidity parameter b must be greater than zero")]
    InvalidB,
    #[msg("Duration must be positive")]
    InvalidDuration,
    #[msg("Invalid owner for the mint account.")]
    InvalidOwner,
    #[msg("Invalid mint account.")]
    InvalidMint,
    #[msg("Market is closed")]
    MarketClosed,
    #[msg("Invalid outcome index")]
    InvalidOutcome,
    #[msg("Must buy at least one share")]
    InvalidShares,
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Underflow occurred")]
    Underflow,
    #[msg("Math error")]
    MathError,
    #[msg("Market not closed yet")]
    MarketNotClosed,
    #[msg("Market already settled")]
    MarketAlreadySettled,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("No fees to withdraw")]
    NoFeesToWithdraw,
    #[msg("No shares to claim")]
    NoSharesToClaim,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    
   
}
