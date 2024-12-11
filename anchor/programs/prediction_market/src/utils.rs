use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, InitializeMint};
use solana_program::pubkey::Pubkey;
use crate::error::CustomError;

/// Calculates the cost based on LMSR formula
pub fn calculate_cost(q: &Vec<u64>, b: u64) -> Result<u64> {
    // Implement the LMSR cost calculation using integer arithmetic
    // Since Rust doesn't have a built-in fixed-point library in the standard library,
    // you'll need to simulate fixed-point by scaling numbers appropriately.
    // For simplicity, let's assume b is scaled appropriately.

    // Placeholder implementation: sum of shares
    let sum: u64 = q.iter().sum();
    Ok(sum * b)
}

/// Calculates the fee based on cost and fee percent
pub fn calculate_fee(cost: u64, fee_percent: u64) -> Result<u64> {
    // fee_percent is expected to be in basis points (e.g., 500 for 5%)
    let fee = (cost.checked_mul(fee_percent).ok_or(CustomError::Overflow)?)
        .checked_div(10000)
        .ok_or(CustomError::Overflow)?;
    Ok(fee)
}
/* 
pub fn create_mint<'info>(
    ctx: Context<info>, CreateMint<'info>>,
    decimals: u8,
) -> Result<Pubkey> {
    let cpi_accounts = InitializeMint {
        mint: ctx.accounts.mint.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::initialize_mint(cpi_ctx, decimals, &ctx.accounts.authority.key(), None)?;

    Ok(ctx.accounts.mint.key())
}

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(init, payer = authority, space = Mint::LEN)]
    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
*/