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
}
