use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Burn};
use crate::state::market::Market;
use crate::error::CustomError;
use crate::constants::TOKEN_DECIMALS;

pub fn handler(
    ctx: Context<ClaimPayout>,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Validations
    require!(market.market_settled, CustomError::MarketNotSettled);
    require!(market.winning_outcome < market.outcomes.len() as u64, CustomError::InvalidOutcome);

    let outcome = &mut market.outcomes[market.winning_outcome as usize];
    let user_shares = ctx.accounts.user_shares.amount;

    require!(user_shares > 0, CustomError::NoSharesToClaim);

    // Calculate payout
    let payout = user_shares.checked_mul(market.payout_per_share).ok_or(CustomError::Overflow)?;

    // Ensure market has sufficient funds
    require!(market.market_maker_funds >= payout, CustomError::InsufficientFunds);

    // Update market funds
    market.market_maker_funds = market.market_maker_funds.checked_sub(payout).ok_or(CustomError::Underflow)?;

    // Burn user's shares
    let cpi_accounts = Burn {
        mint: ctx.accounts.share_mint.to_account_info(),
        from: ctx.accounts.user_shares.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::burn(cpi_ctx, user_shares)?;

    // Transfer payout tokens to user
    let cpi_accounts = Transfer {
        from: ctx.accounts.market_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.market.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, payout)?;

    // Emit event
    msg!("Payout Claimed: {} tokens to user", payout);

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimPayout<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut, has_one = base_token_mint)]
    pub market_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut, has_one = market)]
    pub share_mint: Account<'info, Mint>,

    #[account(mut, has_one = share_mint)]
    pub user_shares: Account<'info, TokenAccount>,

    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}
