use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Token, TokenAccount, Transfer};
use anchor_spl::token:: Mint;
use crate::state::market::Market;
use crate::constants::PAYOUT_PER_SHARE;
use crate::error::CustomError;

pub fn handler(
    ctx: Context<ClaimPayout>,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let user_share_account = &ctx.accounts.user_share_account;
    let winning_outcome = market.winning_outcome;
    
    // Validations
    require!(market.market_settled, CustomError::MarketNotSettled);
    require!(winning_outcome < market.outcomes.len() as u64, CustomError::InvalidOutcome);

    let outcome = &mut market.outcomes[winning_outcome as usize];
    let user_shares = user_share_account.amount;

    require!(user_shares > 0, CustomError::NoSharesToClaim);

    // Calculate payout
    let payout = user_shares.checked_mul(PAYOUT_PER_SHARE).ok_or(CustomError::Overflow)?;

    // Ensure market has sufficient funds
    require!(market.market_maker_funds >= payout, CustomError::InsufficientFunds);

    // Update market funds
    market.market_maker_funds = market.market_maker_funds.checked_sub(payout).ok_or(CustomError::Underflow)?;

    // Burn user's shares
    let cpi_accounts = Burn {
        mint: ctx.accounts.outcome_mint.to_account_info(),
        from: ctx.accounts.user_share_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::burn(cpi_ctx, user_shares)?;

    // Transfer payout tokens to user
    let market_id_bytes: [u8; 8] = market.market_id.to_le_bytes();
    let seeds = &[b"market", &market_id_bytes[..], &[market.bump]];
    let signer_seeds = &[&seeds[..]];

    let refund_transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.market_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: market.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(refund_transfer_ctx, payout)?;
    msg!(
        "Transferred {} tokens from market to user's account",
        payout
    );

    // Emit event
    msg!("Payout Claimed: {} tokens to user", payout);

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimPayout<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub market_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub outcome_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_share_account: Account<'info, TokenAccount>,

    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}
