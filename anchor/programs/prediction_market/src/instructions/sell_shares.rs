use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::market::Market;
use crate::error::CustomError;
use crate::utils::{calculate_cost, calculate_fee};
use crate::constants::SHARES_DECIMALS;
use crate::state::outcome::Outcome;

pub fn handler(
    ctx: Context<SellShares>,
    outcome_index: u64,
    num_shares: u64,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Validations
    require!(!market.market_closed, CustomError::MarketClosed);
    require!(outcome_index < market.outcomes.len() as u64, CustomError::InvalidOutcome);
    require!(num_shares > 0, CustomError::InvalidShares);

    let outcome = &mut market.outcomes[outcome_index as usize];

    // Ensure user has enough shares (Implement share tracking)
    // For simplicity, assume shares are tracked in TokenAccount balances

    // Calculate cost before selling
    let q_before: Vec<u64> = market.outcomes.iter().map(|o| o.total_shares).collect();
    let cost_before = calculate_cost(&q_before, market.b)?;

    // Update shares
    outcome.total_shares = outcome.total_shares.checked_sub(num_shares).ok_or(CustomError::Underflow)?;

    // Calculate cost after selling
    let q_after: Vec<u64> = market.outcomes.iter().map(|o| o.total_shares).collect();
    let cost_after = calculate_cost(&q_after, market.b)?;

    // Cost difference
    let cost_difference = cost_before.checked_sub(cost_after).ok_or(CustomError::MathError)?;

    // Scale cost difference
    let payment = (cost_difference as u128 * 10u128.pow(SHARES_DECIMALS)) as u64; // Adjust scaling as needed

    // Calculate fee
    let fee_amount = calculate_fee(payment, market.fee_percent)?;
    let reinvest_amount = fee_amount / 2;
    let fee_recipient_amount = fee_amount - reinvest_amount;
    let net_payment = payment.checked_sub(fee_amount).ok_or(CustomError::Underflow)?;

    // Transfer tokens from market to seller
    let cpi_accounts = Transfer {
        from: ctx.accounts.market_token_account.to_account_info(),
        to: ctx.accounts.seller_token_account.to_account_info(),
        authority: ctx.accounts.market.to_account_info(), // Must be a signer or PDA
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, net_payment)?;

    // Update market funds
    market.market_maker_funds = market.market_maker_funds.checked_sub(payment).ok_or(CustomError::Underflow)?;
    market.collected_fees = market.collected_fees.checked_add(fee_recipient_amount).ok_or(CustomError::Overflow)?;

    // Emit event
    msg!("Shares Sold: {} shares for outcome {}", num_shares, outcome_index);

    Ok(())
}

#[derive(Accounts)]
pub struct SellShares<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut, has_one = base_token_mint)]
    pub seller_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub market_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
