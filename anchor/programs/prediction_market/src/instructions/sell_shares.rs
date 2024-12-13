// src/instructions/sell_shares.rs

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Burn};
use anchor_spl::token::Mint;
use anchor_spl::token_2022::spl_token_2022::extension::confidential_transfer::instruction;
use crate::state::market::Market;
use crate::error::CustomError;
use crate::utils::{calculate_cost, calculate_fee};
use crate::constants::SHARES_DECIMALS;

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

    msg!("Derived Seller Token Account: {}", ctx.accounts.seller_token_account.key());
    msg!("Derived Market Token Account: {}", ctx.accounts.market_token_account.key());
    msg!("Market Base Token Mint: {}", market.base_token_mint);
    msg!("Instruction Base Token Mint: {}", ctx.accounts.base_token_mint.key());

    // Calculate cost before selling
    let q_before: Vec<u64> = market.outcomes.iter().map(|o| o.total_shares).collect();
    let cost_before = calculate_cost(&q_before, market.b)?;

    // Update shares by subtracting the sold shares
    let outcome = &mut market.outcomes[outcome_index as usize];
    require!(outcome.total_shares >= num_shares, CustomError::InsufficientShares);
    outcome.total_shares = outcome.total_shares.checked_sub(num_shares).ok_or(CustomError::Overflow)?;

    // Calculate cost after selling
    let q_after: Vec<u64> = market.outcomes.iter().map(|o| o.total_shares).collect();
    let cost_after = calculate_cost(&q_after, market.b)?;

    // Cost difference (refund to the user)
    let cost_difference = cost_before.checked_sub(cost_after).ok_or(CustomError::MathError)?;

    // Scale cost difference
    let cost_scaled: u64 = cost_difference; // Already scaled in calculate_cost

    // Calculate fee
    let fee_amount: u64 = calculate_fee(cost_scaled, market.fee_percent)?;
    let reinvest_amount: u64 = fee_amount.checked_div(2).ok_or(CustomError::Overflow)?;
    let fee_recipient_amount: u64 = fee_amount.checked_sub(reinvest_amount).ok_or(CustomError::Overflow)?;
    let net_refund: u64 = cost_scaled.checked_sub(fee_amount).ok_or(CustomError::Overflow)?;

    // Log fee details
    msg!("Fee Amount: {}", fee_amount);
    msg!("Reinvest Amount: {}", reinvest_amount);
    msg!("Fee Recipient Amount: {}", fee_recipient_amount);
    msg!("Net Refund (Cost - Fee): {}", net_refund);

    // Transfer tokens from market to seller
    let cpi_accounts = Transfer {
        from: ctx.accounts.market_token_account.to_account_info(),
        to: ctx.accounts.seller_token_account.to_account_info(),
        authority: ctx.accounts.market.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, net_refund)?;

    // Burn the sold shares from the seller's share account
    let cpi_burn_accounts = Burn {
        mint: ctx.accounts.outcome_mint.to_account_info(),
        to: ctx.accounts.seller_share_account.to_account_info(),
        authority: ctx.accounts.seller.to_account_info(),
    };
    let cpi_burn_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_burn_accounts);
    token::burn(cpi_burn_ctx, num_shares)?;

    // Update market funds
    market.market_maker_funds = market.market_maker_funds.checked_sub(cost_scaled).ok_or(CustomError::Overflow)?;
    market.collected_fees = market.collected_fees.checked_add(fee_recipient_amount).ok_or(CustomError::Overflow)?;

    // Emit event (using logs for now)
    msg!("Shares Sold: {} shares for outcome {}", num_shares, outcome_index);

    Ok(())
}

#[derive(Accounts)]
#[instruction(outcome_index: u64, num_shares: u64)]
pub struct SellShares<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    /// The seller's token account for the base token
    #[account(
        mut,
        associated_token::mint = base_token_mint,
        associated_token::authority = seller
    )]
    pub seller_token_account: Account<'info, TokenAccount>,

    /// The market's token account for the base token
    #[account(
        mut,
        associated_token::mint = base_token_mint,
        associated_token::authority = market
    )]
    pub market_token_account: Account<'info, TokenAccount>,

    /// The seller's token account for the outcome shares
    #[account(
        mut,
        associated_token::mint = outcome_mint,
        associated_token::authority = seller
    )]
    pub seller_share_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        constraint = base_token_mint.key() == market.base_token_mint,
        address = market.base_token_mint
    )]
    pub base_token_mint: Account<'info, Mint>,

    #[account(
        constraint = outcome_mint.key() == market.outcomes[outcome_index as usize].mint,
    )]
    pub outcome_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
