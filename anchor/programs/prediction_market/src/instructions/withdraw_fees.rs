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
    msg!("Fees to withdraw: {}", fees);

    // Transfer payout tokens to user
    let market_id_bytes: [u8; 8] = market.market_id.to_le_bytes();
    let seeds = &[b"market", &market_id_bytes[..], &[market.bump]];
    let signer_seeds = &[&seeds[..]];

    let refund_transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.market_token_account.to_account_info(),
            to: ctx.accounts.fee_recipient_token_account.to_account_info(),
            authority: market.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(refund_transfer_ctx, fees)?;
    msg!(
        "Transferred {} tokens from market to fee recipients's token  account",
        fees
    );
    
    // Emit event
    msg!("Fees Withdrawn: {} tokens to fee recipient", fees);

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub market_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub fee_recipient_token_account: Account<'info, TokenAccount>,

    pub fee_recipient: Signer<'info>,

    pub token_program: Program<'info, Token>,
}
