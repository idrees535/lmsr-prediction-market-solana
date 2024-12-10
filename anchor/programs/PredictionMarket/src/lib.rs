#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
//use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};
//use fixed::types::I64F64;
use anchor_spl::token::spl_token;

use solana_program::entrypoint::ProgramResult;

use crate::state::market::Market; // Import Market from the state/market.rs file
use crate::state::outcome::Outcome; // Import Outcome from state/outcome.rs

pub mod constants;
pub mod error;
//pub mod instructions;
pub mod state;

//use instructions::*;
//use instructions::create_market::CreateMarket;


declare_id!("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");


#[program]
pub mod PredictionMarket {
    use super::*;
    pub fn create_market(
        ctx: Context<CreateMarket>,
        market_id: u64,
        title: String,
        outcomes: Vec<String>,
        oracle: Pubkey,
        b: u64,
        duration: i64,
        fee_percent: u64,
        fee_recipient: Pubkey,
        initial_funds: u64,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        let clock = Clock::get()?;

        // Basic validations
        require!(outcomes.len() > 0, CustomError::NoOutcomes);
        require!(b > 0, CustomError::InvalidB);
        require!(duration > 0, CustomError::InvalidDuration);


        require!(
        ctx.accounts.base_token_mint.owner == &spl_token::ID,
        CustomError::InvalidOwner
    );

        market.market_id = market_id;
        market.title = title;
        market.oracle = oracle;
        market.b = b;
        market.fee_percent = fee_percent;
        market.fee_recipient = fee_recipient;
        market.end_timestamp = clock.unix_timestamp + duration;
        market.market_closed = false;
        market.market_settled = false;
        market.winning_outcome = 0;
        market.market_maker_funds = initial_funds;
        market.initial_funds = initial_funds;
        market.collected_fees = 0;

        // In a real scenario, you should have a known base token mint.
        // For demonstration, we store a placeholder value:

        market.base_token_mint = ctx.accounts.base_token_mint.key();

        // Initialize outcomes
        let mut outcomes_structs: Vec<Outcome> = Vec::with_capacity(outcomes.len());
        for outcome_name in outcomes.iter() {
            // If you'd like to create a mint for each outcome, do it here:
            // - create_mint CPI call
            // - store mint pubkey in outcome.mint
            // For now, we’ll just store a placeholder.
            let dummy_mint = Pubkey::default(); 
            outcomes_structs.push(Outcome {
                name: outcome_name.clone(),
                total_shares: 0,
                mint: dummy_mint,
            });
        }
        market.outcomes = outcomes_structs;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateMarket<'info> {
    #[account(
        init,
        payer = user,
        space = 8 +  // Discriminator
        8 +  // market_id
        4 + 50 + // title (max length + 4 bytes for length prefix)
        32 + // oracle
        8 + // b
        8 + // fee_percent
        32 + // fee_recipient
        // outcomes vector: depends on number of outcomes, handle in runtime
        8 + // end_timestamp
        1 + // market_closed (bool)
        1 + // market_settled (bool)
        8 + // winning_outcome
        8 + // market_maker_funds
        8 + // initial_funds
        8 + // collected_fees
        32   // base_token_mint
        // Add buffer for dynamic outcomes
    )]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub user: Signer<'info>,

   ///CHECK: The base_token_mint is provided by the user. We must check its validity at runtime.
    #[account()]
    pub base_token_mint: UncheckedAccount<'info>,

    #[account(address = anchor_lang::solana_program::system_program::ID)]
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[error_code]
pub enum CustomError {
    #[msg("At least one outcome is required")]
    NoOutcomes,
    #[msg("Liquidity parameter b must be greater than zero")]
    InvalidB,
    #[msg("Duration must be positive")]
    InvalidDuration,
    #[msg("Invalid owner for the mint account.")]
    InvalidOwner,
}


/*
    pub fn buy_shares(ctx: Context<BuyShares>, outcome_index: u8, num_shares: u64) -> Result<()> {
        instructions::buy_shares::handler(ctx, outcome_index, num_shares)
    }

    pub fn sell_shares(ctx: Context<SellShares>, outcome_index: u8, num_shares: u64) -> Result<()> {
        instructions::sell_shares::handler(ctx, outcome_index, num_shares)
    }

    pub fn set_outcome(ctx: Context<SetOutcome>, outcome: u8) -> Result<()> {
        instructions::set_outcome::handler(ctx, outcome)
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        instructions::withdraw_fees::handler(ctx)
    }

    pub fn create_market(ctx: Context<CreateMarket>, params: CreateMarketParams) -> ProgramResult {
        let market = &mut ctx.accounts.market;
        market.market_id = params.market_id;
        market.title = params.title;
        market.b = params.b;
        market.oracle = params.oracle;
        market.market_end_time = Clock::get()?.unix_timestamp + params.duration;
        market.market_closed = false;
        market.market_settled = false;
        market.fee_percent = params.fee_percent;
        market.fee_recipient = params.fee_recipient;
        market.collected_fees = 0;
        market.token_mint = params.token_mint;
        market.initial_funds = params.initial_funds;
        market.market_maker_funds = params.initial_funds;

        // Initialize outcomes
        for outcome_name in params.outcomes.iter() {
            market.outcomes.push(Outcome {
                name: outcome_name.clone(),
                total_shares: 0,
            });
        }

        // Initialize the market maker's token account (PDA)
        let cpi_accounts = anchor_spl::associated_token::CreateAssociatedTokenAccount {
            payer: ctx.accounts.user.to_account_info(),
            associated_token: ctx.accounts.market_maker_token_account.to_account_info(),
            authority: ctx.accounts.market.to_account_info(),
            mint: ctx.accounts.token_mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let cpi_program = ctx.accounts.associated_token_program.to_account_info();
        anchor_spl::associated_token::create_associated_token_account(CpiContext::new(
            cpi_program,
            cpi_accounts,
        ))?;

        // Mint initial funds to market_maker_token_account
        if market.initial_funds > 0 {
            let cpi_accounts_mint = MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.market_maker_token_account.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            };
            let seeds = &[b"market".as_ref(), &market.market_id.to_le_bytes()];
            let signer = &[&seeds[..]];
            let cpi_ctx_mint = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_mint,
                signer,
            );
            token::mint_to(cpi_ctx_mint, market.initial_funds)?;
        }

        // Emit MarketCreated event
        emit!(MarketCreated {
            market_id: params.market_id,
            title: params.title.clone(),
            creator: ctx.accounts.user.key(),
        });

        Ok(())
    }

    pub fn buy_shares(
        ctx: Context<BuyShares>,
        outcome_index: u8,
        num_shares: u64,
    ) -> ProgramResult {
        let market = &mut ctx.accounts.market;

        // Validate market state
        if market.market_closed {
            return Err(ErrorCode::MarketClosed.into());
        }

        if outcome_index as usize >= market.outcomes.len() {
            return Err(ErrorCode::InvalidOutcomeIndex.into());
        }

        // Calculate cost using the math library
        let cost = calculate_cost(&market.outcomes, market.b)?;

        // Transfer tokens from user to market maker
        let cpi_accounts_transfer = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.market_maker_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program_transfer = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_transfer = CpiContext::new(cpi_program_transfer, cpi_accounts_transfer);
        token::transfer(cpi_ctx_transfer, cost)?;

        // Mint share tokens to the user
        let cpi_accounts_mint = MintTo {
            mint: ctx.accounts.outcome_mint.to_account_info(),
            to: ctx.accounts.user_share_token_account.to_account_info(),
            authority: ctx.accounts.market.to_account_info(),
        };
        let seeds = &[b"market".as_ref(), &market.market_id.to_le_bytes()];
        let signer = &[&seeds[..]];
        let cpi_ctx_mint = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts_mint,
            signer,
        );
        token::mint_to(cpi_ctx_mint, num_shares)?;

        // Update market state
        market.outcomes[outcome_index as usize].total_shares = market.outcomes
            [outcome_index as usize]
            .total_shares
            .checked_add(num_shares)
            .ok_or(ErrorCode::Overflow)?;

        // Handle fee calculation and accumulation using fixed-point arithmetic
        let cost_fixed = I64F64::from_num(cost);
        let fee_percent_fixed = I64F64::from_num(market.fee_percent) / I64F64::from_num(100u64);
        let fee_fixed = cost_fixed * fee_percent_fixed;
        let fee = fee_fixed.to_num::<u64>();

        market.collected_fees = market
            .collected_fees
            .checked_add(fee)
            .ok_or(ErrorCode::Overflow)?;

        // Emit SharesPurchased event
        emit!(SharesPurchased {
            user: ctx.accounts.user.key(),
            outcome_index,
            num_shares,
            cost,
            fee,
        });

        Ok(())
    }

    pub fn sell_shares(
        ctx: Context<SellShares>,
        outcome_index: u8,
        num_shares: u64,
    ) -> ProgramResult {
        let market = &mut ctx.accounts.market;

        // Ensure the market is open
        if market.market_closed {
            return Err(ErrorCode::MarketClosed.into());
        }

        if outcome_index as usize >= market.outcomes.len() {
            return Err(ErrorCode::InvalidOutcomeIndex.into());
        }

        // Calculate payment using the math library
        let payment = calculate_payment(&market.outcomes, market.b)?;

        // Burn share tokens from the user
        let cpi_accounts_burn = Burn {
            mint: ctx.accounts.outcome_mint.to_account_info(),
            from: ctx.accounts.user_share_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program_burn = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_burn = CpiContext::new(cpi_program_burn, cpi_accounts_burn);
        token::burn(cpi_ctx_burn, num_shares)?;

        // Transfer tokens from market maker to user
        let cpi_accounts_transfer = Transfer {
            from: ctx.accounts.market_maker_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.market.to_account_info(),
        };
        let cpi_program_transfer = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_transfer = CpiContext::new(cpi_program_transfer, cpi_accounts_transfer);
        token::transfer(cpi_ctx_transfer, payment)?;

        // Update market state
        market.outcomes[outcome_index as usize].total_shares = market.outcomes
            [outcome_index as usize]
            .total_shares
            .checked_sub(num_shares)
            .ok_or(ErrorCode::Underflow)?;

        // Handle fee calculation and accumulation using fixed-point arithmetic
        let payment_fixed = I64F64::from_num(payment);
        let fee_percent_fixed = I64F64::from_num(market.fee_percent) / I64F64::from_num(100u64);
        let fee_fixed = payment_fixed * fee_percent_fixed;
        let fee = fee_fixed.to_num::<u64>();

        market.collected_fees = market
            .collected_fees
            .checked_add(fee)
            .ok_or(ErrorCode::Overflow)?;

        // Emit SharesSold event
        emit!(SharesSold {
            user: ctx.accounts.user.key(),
            outcome_index,
            num_shares,
            payment,
            fee,
        });

        Ok(())
    }

    pub fn set_outcome(ctx: Context<SetOutcome>, outcome: u8) -> ProgramResult {
        let market = &mut ctx.accounts.market;

        // Ensure only the oracle can set the outcome
        if market.oracle != ctx.accounts.oracle.key() {
            return Err(ErrorCode::Unauthorized.into());
        }

        // Ensure the market is closed and not yet settled
        if !market.market_closed {
            return Err(ErrorCode::MarketNotClosed.into());
        }

        if market.market_settled {
            return Err(ErrorCode::AlreadySettled.into());
        }

        // Validate the outcome index
        if outcome as usize >= market.outcomes.len() {
            return Err(ErrorCode::InvalidOutcomeIndex.into());
        }

        // Set the winning outcome
        market.winning_outcome = outcome;
        market.market_settled = true;

        // Emit OutcomeSet event
        emit!(OutcomeSet {
            market_id: market.market_id,
            winning_outcome: outcome,
        });

        Ok(())
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> ProgramResult {
        let factory = &mut ctx.accounts.factory;

        // Ensure there are fees to withdraw
        if factory.collected_fees == 0 {
            return Err(ErrorCode::NoFees.into());
        }

        // Transfer collected fees to fee recipient
        let cpi_accounts_transfer = Transfer {
            from: ctx.accounts.fee_account.to_account_info(),
            to: ctx.accounts.fee_recipient_token_account.to_account_info(),
            authority: ctx.accounts.factory.to_account_info(),
        };
        let seeds = &[b"factory".as_ref()];
        let signer = &[&seeds[..]];
        let cpi_ctx_transfer = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts_transfer,
            signer,
        );
        token::transfer(cpi_ctx_transfer, factory.collected_fees)?;

        // Reset collected fees
        factory.collected_fees = 0;

        // Emit FeesWithdrawn event
        emit!(FeesWithdrawn {
            fee_recipient: ctx.accounts.fee_recipient.key(),
            amount: factory.collected_fees,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateMarket<'info> {
    #[account(init, payer = user, space = 8 + Market::LEN)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = market,
    )]
    pub market_maker_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct Market {
    pub market_id: u64,
    pub title: String,
    pub outcomes: Vec<Outcome>,
    pub b: u64,
    pub oracle: Pubkey,
    pub market_end_time: i64,
    pub market_closed: bool,
    pub market_settled: bool,
    pub winning_outcome: u8,
    pub fee_percent: u8,
    pub fee_recipient: Pubkey,
    pub collected_fees: u64,
    pub token_mint: Pubkey,
    pub initial_funds: u64,
    pub market_maker_funds: u64,
}

impl Market {
    // Assuming maximum 10 outcomes and maximum title length of 100 bytes
    const MAX_OUTCOMES: usize = 10;
    const MAX_TITLE_LENGTH: usize = 100;
    const LEN: usize = 8 // Discriminator
        + 8 // market_id
        + 4 + Market::MAX_TITLE_LENGTH // title (Vec<u8>, prefix with u32 length)
        + 8 // b
        + 32 // oracle
        + 8 // market_end_time
        + 1 // market_closed
        + 1 // market_settled
        + 1 // winning_outcome
        + 1 // fee_percent
        + 32 // fee_recipient
        + 8 // collected_fees
        + 32 // token_mint
        + 8 // initial_funds
        + 8 // market_maker_funds
        + (Market::MAX_OUTCOMES * Outcome::LEN); // outcomes
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Outcome {
    pub name: String,
    pub total_shares: u64,
}

impl Outcome {
    const LEN: usize = 4 + 50 + 8; // String length prefix + max 50 bytes for name + total_shares
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateMarketParams {
    pub market_id: u64,
    pub title: String,
    pub outcomes: Vec<String>,
    pub b: u64,
    pub duration: i64,
    pub fee_percent: u8,
    pub oracle: Pubkey,
    pub fee_recipient: Pubkey,
    pub token_mint: Pubkey,
    pub initial_funds: u64,
}

#[derive(Accounts)]
pub struct BuyShares<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_share_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub outcome_mint: Account<'info, Mint>,
    #[account(mut)]
    pub market_maker_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct SellShares<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_share_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub outcome_mint: Account<'info, Mint>,
    #[account(mut)]
    pub market_maker_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SellSharesParams {
    pub num_shares: u64,
    // Add additional fields if necessary
}

#[derive(Accounts)]
pub struct SetOutcome<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    pub oracle: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(mut)]
    pub factory: Account<'info, Factory>,
    #[account(mut)]
    pub fee_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_recipient_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Factory {
    pub active_markets: Vec<Pubkey>,
    pub collected_fees: u64,
}

impl Factory {
    // Assuming maximum 100 markets
    const MAX_MARKETS: usize = 100;
    const LEN: usize = 8 // Discriminator
        + (32 * Factory::MAX_MARKETS) // active_markets
        + 8; // collected_fees
}

#[event]
pub struct MarketCreated {
    pub market_id: u64,
    pub title: String,
    pub creator: Pubkey,
}

#[event]
pub struct SharesPurchased {
    pub user: Pubkey,
    pub outcome_index: u8,
    pub num_shares: u64,
    pub cost: u64,
    pub fee: u64,
}

#[event]
pub struct SharesSold {
    pub user: Pubkey,
    pub outcome_index: u8,
    pub num_shares: u64,
    pub payment: u64,
    pub fee: u64,
}

#[event]
pub struct OutcomeSet {
    pub market_id: u64,
    pub winning_outcome: u8,
}

#[event]
pub struct FeesWithdrawn {
    pub fee_recipient: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Market is closed")]
    MarketClosed,
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Underflow occurred")]
    Underflow,
    #[msg("Unauthorized action")]
    Unauthorized,
    #[msg("Invalid outcome index")]
    InvalidOutcomeIndex,
    #[msg("Calculation error")]
    CalculationError,
    #[msg("Already settled")]
    AlreadySettled,
    #[msg("Market not closed")]
    MarketNotClosed,
    #[msg("No fees to withdraw")]
    NoFees,
}

fn fixed_exp(x: I64F64) -> I64F64 {
    // Simple implementation using a Taylor series expansion or other method
    // For demonstration purposes, this is overly simplified and not suitable for production
    let one = I64F64::from_num(1);
    let x_squared = x * x;
    one + x + (x_squared / I64F64::from_num(2))
}

fn fixed_ln(x: I64F64) -> I64F64 {
    // Simple implementation using a logarithm approximation
    // For demonstration purposes, this is overly simplified and not suitable for production
    // Note: ln(1 + x) ≈ x - x^2/2 + x^3/3 - x^4/4 + ...
    let x_minus_one = x - I64F64::from_num(1);
    x_minus_one - (x_minus_one * x_minus_one / I64F64::from_num(2))
}

fn calculate_cost(outcomes: &Vec<Outcome>, b: u64) -> Result<u64> {
    let b_fixed = I64F64::from_num(b);
    let mut sum_exp = I64F64::ZERO;

    for outcome in outcomes.iter() {
        let qi = I64F64::from_num(outcome.total_shares);
        let qi_div_b = qi / b_fixed;
        sum_exp += fixed_exp(qi_div_b);
    }

    let cost_fixed = b_fixed * fixed_ln(sum_exp);
    let cost = cost_fixed.to_num::<u64>();

    Ok(cost)
}


fn calculate_payment(outcomes: &Vec<Outcome>, b: u64) -> Result<u64> {
    // Implement the payment calculation logic based on LMSR
    // Placeholder implementation
    let payment = 1000u64; // Replace with actual calculation
    Ok(payment)
}

/*
    pub fn sell_shares(ctx: Context<SellShares>, params: SellSharesParams) -> ProgramResult {
        // Implement share selling logic
    }

    pub fn close_market(ctx: Context<CloseMarket>) -> ProgramResult {
        // Implement market closing logic
    }

    pub fn set_outcome(ctx: Context<SetOutcome>, outcome: u8) -> ProgramResult {
        // Implement outcome setting logic
    }

    pub fn claim_payout(ctx: Context<ClaimPayout>) -> ProgramResult {
        // Implement payout claiming logic
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> ProgramResult {
        // Implement fee withdrawal logic
    }
}
*/
*/