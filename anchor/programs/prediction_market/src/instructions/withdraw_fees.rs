use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::market::Market;
use crate::error::CustomError;

pub fn handler(
    ctx: Context<WithdrawFees>,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Validations
    require!(ctx.accounts.fee_recipient.key() == market.fee_recipient, CustomError::Unauthorized);

    let fees = market.collected_fees;
    require!(fees > 0, CustomError::NoFeesToWithdraw);

    // Reset collected fees
    market.collected_fees = 0;

    // Transfer fees to fee recipient
    let cpi_accounts = Transfer {
        from: ctx.accounts.market_token_account.to_account_info(),
        to: ctx.accounts.fee_recipient_token_account.to_account_info(),
        authority: ctx.accounts.market.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, fees)?;

    // Emit event
    msg!("Fees Withdrawn: {} tokens to fee recipient", fees);

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut, has_one = base_token_mint)]
    pub market_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub fee_recipient_token_account: Account<'info, TokenAccount>,

    pub fee_recipient: Signer<'info>,

    pub token_program: Program<'info, Token>,
}
