// src/instructions/sell_shares.rs

use crate::error::CustomError;
use crate::state::market::Market;
use crate::utils::{calculate_cost, calculate_fee};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};


pub fn handler(ctx: Context<SellShares>, outcome_index: u64, num_shares: u64) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let outcome_mint = &ctx.accounts.outcome_mint;
    let buyer_share_account = &ctx.accounts.buyer_share_account;

    // Validation checks
    require!(!market.market_closed, CustomError::MarketClosed);
    require!(
        outcome_index < market.outcomes.len() as u64,
        CustomError::InvalidOutcome
    );
    require!(num_shares > 0, CustomError::InvalidShares);

    // Ensure the user's share account is associated with the correct mint
    require!(
        buyer_share_account.mint == outcome_mint.key(),
        CustomError::InvalidMint
    );
    require!(
        buyer_share_account.owner == ctx.accounts.seller.key(),
        CustomError::InvalidOwner
    );

    // Ensure the user has enough shares to sell
    let user_share_balance = buyer_share_account.amount;
    require!(
        user_share_balance >= num_shares,
        CustomError::InsufficientShares
    );

    // Calculate LMSR refund
    let q_before: Vec<u64> = market.outcomes.iter().map(|o| o.total_shares).collect();
    let cost_before = calculate_cost(&q_before, market.b)?;

    let outcome = &mut market.outcomes[outcome_index as usize];
    outcome.total_shares = outcome
        .total_shares
        .checked_sub(num_shares)
        .ok_or(CustomError::MathError)?;

    let q_after: Vec<u64> = market.outcomes.iter().map(|o| o.total_shares).collect();
    let cost_after = calculate_cost(&q_after, market.b)?;

    let refund_amount = cost_before
        .checked_sub(cost_after)
        .ok_or(CustomError::MathError)? as u64;

    // Calculate fee
    let fee_amount: u64 = calculate_fee(refund_amount, market.fee_percent)?;
    let reinvest_amount: u64 = fee_amount.checked_div(2).ok_or(CustomError::Overflow)?;
    let fee_recipient_amount: u64 = fee_amount
        .checked_sub(reinvest_amount)
        .ok_or(CustomError::Overflow)?;
    let net_refund: u64 = refund_amount
        .checked_sub(fee_amount)
        .ok_or(CustomError::Overflow)?;
    msg!("Fee Amount: {}", fee_amount);
    msg!("Reinvest Amount: {}", reinvest_amount);
    msg!("Fee Recipient Amount: {}", fee_recipient_amount);
    msg!("Net Refund (Cost - Fee): {}", net_refund);
    msg!(
        "User share balance before: {}, in user share account: {}",
        buyer_share_account.amount,
        buyer_share_account.key()
    );

    // Burn shares from user's account
    let market_id_bytes: [u8; 8] = market.market_id.to_le_bytes();
    let seeds = &[b"market", &market_id_bytes[..], &[market.bump]];
    let signer_seeds = &[&seeds[..]];
    msg!("Burn operation initiated by: {}", ctx.accounts.seller.key());
    msg!(
        "Mint authority for outcome_mint: {:?}",
        outcome_mint.mint_authority.unwrap()
    );


    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Burn {
            mint: outcome_mint.to_account_info(),
            from: buyer_share_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(), // User is the authority
        },
    );
    token::burn(burn_ctx, num_shares)?;

    let updated_buyer_share_account = token::accessor::amount(&buyer_share_account.to_account_info())?;
    msg!("Updated user_share_balance: {}", updated_buyer_share_account);

    msg!("Burned {} shares from user's account", num_shares);

    // Transfer refund from market to user
    let refund_transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.market_token_account.to_account_info(),
            to: ctx.accounts.seller_token_account.to_account_info(),
            authority: market.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(refund_transfer_ctx, net_refund)?;
    msg!(
        "Transferred {} tokens from market to user's account",
        net_refund
    );

    // Update market funds
    market.market_maker_funds = market
        .market_maker_funds
        .checked_sub(refund_amount)
        .ok_or(CustomError::Overflow)?;

    // Emit event
    msg!(
        "Shares Sold: {} shares for outcome {} with refund {}",
        num_shares,
        outcome_index,
        refund_amount
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(outcome_index: u64, num_shares: u64)]
pub struct SellShares<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        associated_token::mint = outcome_mint,
        associated_token::authority = seller
    )]
    pub buyer_share_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = outcome_mint.key() == market.outcomes[outcome_index as usize].mint
    )]
    pub outcome_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = market.base_token_mint,
        associated_token::authority = seller
    )]
    pub seller_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = market.base_token_mint,
        associated_token::authority = market
    )]
    pub market_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

