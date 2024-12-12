use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_spl::token::Mint;
use crate::state::market::Market;
use crate::error::CustomError;
use crate::utils::{calculate_cost, calculate_fee};
use crate::constants::SHARES_DECIMALS;
//use crate::state::outcome::Outcome;

pub fn handler(
    ctx: Context<BuyShares>,
    outcome_index: u64,
    num_shares: u64,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Validations
    require!(!market.market_closed, CustomError::MarketClosed);
    require!(outcome_index < market.outcomes.len() as u64, CustomError::InvalidOutcome);
    require!(num_shares > 0, CustomError::InvalidShares);

    // Validate that the buyer's token account mint matches the market's base token mint
    // require!(
    //     ctx.accounts.buyer_token_account.mint == market.base_token_mint,
    //     CustomError::InvalidMint
    // );
//msg!("Derived Buyer Token Account: {}", ctx.accounts.buyer_token_account.key());
msg!("Derived Market Token Account: {}", ctx.accounts.market_token_account.key());
msg!("Market Base Token Mint: {}", market.base_token_mint);
msg!("Instruction Base Token Mint: {}", ctx.accounts.base_token_mint.key());



    // Calculate cost before purchase
    let q_before: Vec<u64> = market.outcomes.iter().map(|o| o.total_shares).collect();
    let cost_before = calculate_cost(&q_before, market.b)?;

    let outcome = &mut market.outcomes[outcome_index as usize];
    // Update shares
    outcome.total_shares = outcome.total_shares.checked_add(num_shares).ok_or(CustomError::Overflow)?;

    // Calculate cost after purchase
    let q_after: Vec<u64> = market.outcomes.iter().map(|o| o.total_shares).collect();
    let cost_after = calculate_cost(&q_after, market.b)?;

    // Cost difference
    let cost_difference = cost_after.checked_sub(cost_before).ok_or(CustomError::MathError)?;

    // Scale cost difference
    let cost = (cost_difference as u128 * 10u128.pow(SHARES_DECIMALS)) as u64; // Adjust scaling as needed

    // Calculate fee
    let fee_amount = calculate_fee(cost, market.fee_percent)?;
    let reinvest_amount = fee_amount / 2;
    let fee_recipient_amount = fee_amount - reinvest_amount;
    let net_cost = cost.checked_add(fee_amount).ok_or(CustomError::Overflow)?;

    // Transfer tokens from buyer to market
    let cpi_accounts = Transfer {
        from: ctx.accounts.buyer_token_account.to_account_info(),
        to: ctx.accounts.market_token_account.to_account_info(),
        authority: ctx.accounts.buyer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, net_cost)?;

    // Update market funds
    market.market_maker_funds = market.market_maker_funds.checked_add(cost).ok_or(CustomError::Overflow)?;
    market.collected_fees = market.collected_fees.checked_add(fee_recipient_amount).ok_or(CustomError::Overflow)?;

    // Emit event (if using events in Solana; otherwise, use logs)
    msg!("Shares Purchased: {} shares for outcome {}", num_shares, outcome_index);

    Ok(())
}

#[derive(Accounts)]
pub struct BuyShares<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    //#[account(mut)]
    #[account(
        //mut,
        init_if_needed,
        payer = buyer,
        associated_token::mint = base_token_mint,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    //#[account(mut)]
    #[account(
        //mut,
        init_if_needed,
        payer = buyer,
        associated_token::mint = base_token_mint,
        associated_token::authority = market
    )]
    pub market_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        constraint = base_token_mint.key() == market.base_token_mint,
        address = market.base_token_mint
    )]
    pub base_token_mint: Account<'info, Mint>,

   // #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,

    //#[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
