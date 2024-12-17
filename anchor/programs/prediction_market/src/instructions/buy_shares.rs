use crate::constants::SHARES_DECIMALS;
use crate::error::CustomError;
use crate::state::market::Market;
use crate::state::outcome;
use crate::utils::{calculate_cost, calculate_fee};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, MintTo, Token, TokenAccount, Transfer};
use anchor_spl::token::{spl_token, Mint};
//use crate::state::outcome::Outcome;

pub fn handler(ctx: Context<BuyShares>, outcome_index: u64, num_shares: u64) -> Result<()> {
    msg!("Buy Shares");
    let market = &mut ctx.accounts.market;
    
    // Validations
    require!(!market.market_closed, CustomError::MarketClosed);
    require!(
        outcome_index < market.outcomes.len() as u64,
        CustomError::InvalidOutcome
    );
    require!(num_shares > 0, CustomError::InvalidShares);

    // Calculate cost before purchase
    let q_before: Vec<u64> = market.outcomes.iter().map(|o| o.total_shares).collect();
    let cost_before = calculate_cost(&q_before, market.b)?;

    let outcome = &mut market.outcomes[outcome_index as usize];
    // Update shares
    outcome.total_shares = outcome
        .total_shares
        .checked_add(num_shares)
        .ok_or(CustomError::Overflow)?;

    // Calculate cost after purchase
    let q_after: Vec<u64> = market.outcomes.iter().map(|o| o.total_shares).collect();
    let cost_after = calculate_cost(&q_after, market.b)?;

    // Cost difference
    let cost_difference = cost_after
        .checked_sub(cost_before)
        .ok_or(CustomError::MathError)?;

    // Scale cost difference
    let cost: u64 = (cost_difference as u128 * 10u128.pow(SHARES_DECIMALS)) as u64; // Adjust scaling as needed

    // Calculate fee
    let fee_amount: u64 = calculate_fee(cost, market.fee_percent)?;
    let reinvest_amount: u64 = fee_amount / 2;
    let fee_recipient_amount: u64 = fee_amount - reinvest_amount;
    let net_cost: u64 = cost.checked_add(fee_amount).ok_or(CustomError::Overflow)?;


    msg!("transferring tokens form buyer to market");

    // Transfer tokens from buyer to market
    let cpi_accounts = Transfer {
        from: ctx.accounts.buyer_token_account.to_account_info(),
        to: ctx.accounts.market_token_account.to_account_info(),
        authority: ctx.accounts.buyer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, net_cost)?;


    // Mint shares to the user's associated sahre token account
    msg!("Minting shares to buyer share account");
    let outcome_mint = &ctx.accounts.outcome_mint;
    let buyer_share_account = &ctx.accounts.buyer_share_account;
    

     msg!(
        "Outcome Mint Authority: {:?}",
        outcome_mint.mint_authority.unwrap()
    );
    msg!("Market PDA (On-chain): {}", market.key());
    

    require!(
        outcome_mint.mint_authority.unwrap() == market.key(),
        CustomError::InvalidMintAuthority
    );
    msg!("Mint authority check passsed");
     msg!("Buyer Share Account Mint: {}", buyer_share_account.mint);
    msg!("Outcome Mint: {}", outcome_mint.key());
   
    require!(
        buyer_share_account.mint == outcome_mint.key(),
        CustomError::InvalidMint
    );
    msg!("Mint check passsed");
    msg!("Buyer Share Account Owner: {}", buyer_share_account.owner);
    msg!("Buyer: {}", ctx.accounts.buyer.key());
    require!(
        buyer_share_account.owner == ctx.accounts.buyer.key(),
        CustomError::InvalidOwner
    );
     msg!("others also check passsed");


    // let (pda, bump) = Pubkey::find_program_address(
    //     &[b"market", market.market_id.to_le_bytes().as_ref()],
    //     ctx.program_id,
    // );


    let market_id_bytes: [u8; 8] = market.market_id.to_le_bytes();
    let seeds = &[b"market", &market_id_bytes[..], &[market.bump]];
    
    let signer_seeds = &[&seeds[..]];

    let cpi_mint_to_accounts = MintTo {
        mint: outcome_mint.to_account_info(),
        to: buyer_share_account.to_account_info(),
        authority: market.to_account_info(), // Market is the mint authority
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_mint_to_accounts,
        signer_seeds,
    );

    token::mint_to(cpi_ctx, num_shares)?;

    msg!("Shares minted to buyer share account");


    ////
    // let cpi_mint_to_accounts = MintTo {
    //     mint: outcome_mint.to_account_info(),
    //     to: buyer_share_account.to_account_info(),
    //     authority: market.to_account_info(),
    // };
    // let cpi_mint_to_ctx = CpiContext::new(
    //     ctx.accounts.token_program.to_account_info(),
    //     cpi_mint_to_accounts,
    // );
    // token::mint_to(cpi_mint_to_ctx, num_shares)?;

    // Update market funds
    market.market_maker_funds = market
        .market_maker_funds
        .checked_add(cost)
        .ok_or(CustomError::Overflow)?;
    market.collected_fees = market
        .collected_fees
        .checked_add(fee_recipient_amount)
        .ok_or(CustomError::Overflow)?;

    // Emit event (if using events in Solana; otherwise, use logs)
    msg!(
        "Shares Purchased: {} shares for outcome {}",
        num_shares,
        outcome_index
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(outcome_index: u64, num_shares: u64)]
pub struct BuyShares<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

   
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = base_token_mint,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    #[account(
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

    
    #[account(
        mut,
         //init_if_needed,
        //  payer = buyer,
        //  associated_token::mint = outcome_mint,
        //  associated_token::authority = buyer
     )]
    pub buyer_share_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        // constraint = outcome_mint.key() == market.outcomes[outcome_index as usize].mint,
        // address = market.outcomes[outcome_index as usize].mint
    )]
    pub outcome_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
