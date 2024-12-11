

#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
//use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
//use fixed::types::I64F64;
use anchor_spl::token::spl_token;
use solana_program::program_pack::Pack;

use solana_program::entrypoint::ProgramResult;

use crate::state::market::Market; // Import Market from the state/market.rs file
use crate::state::outcome::Outcome;
use crate::error::CustomError;  // Import Outcome from state/outcome.rs

pub mod constants;
pub mod error;
//pub mod instructions;
pub mod state;

//use instructions::*;
//use instructions::create_market::CreateMarket;

declare_id!("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");

#[program]
pub mod prediction_market {
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

        // Derive the PDA and get the bump
        let (_market_pda, _bump) = Pubkey::find_program_address(
            &[b"market", market_id.to_le_bytes().as_ref()],
            ctx.program_id,
        );

        // Basic validations
        require!(outcomes.len() > 0, CustomError::NoOutcomes);
        require!(b > 0, CustomError::InvalidB);
        require!(duration > 0, CustomError::InvalidDuration);
        require!(
            *ctx.accounts.base_token_mint.owner == spl_token::ID,
            CustomError::InvalidOwner
        );

        require!(
            ctx.accounts.base_token_mint.to_account_info().data_len()
                == spl_token::state::Mint::LEN,
            CustomError::InvalidMint
        );

        // Deserialize the mint to ensure it follows SPL Mint structure
        let mint = spl_token::state::Mint::unpack(
            &ctx.accounts.base_token_mint.to_account_info().data.borrow(),
        )?;
        msg!("Mint supply: {}", mint.supply); // Example: log the total supply

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
        market.market_id = market_id;

        Ok(())
    }
}       

// Define the CreateMarket instruction
#[derive(Accounts)]
#[instruction(market_id: u64, title: String, outcomes: Vec<String>, oracle: Pubkey, b: u64, duration: i64, fee_percent: u64, fee_recipient: Pubkey, initial_funds: u64)]
pub struct CreateMarket<'info> {
    #[account(
        init,
        seeds = [b"market", market_id.to_le_bytes().as_ref()],
        bump,
        payer = user,
        space = 8 +  Market::INIT_SPACE,  
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
*/
