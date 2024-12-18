#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

pub use crate::instructions::*;

declare_id!("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");

#[program]
pub mod prediction_market {
    use super::*;

    pub fn create_market<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateMarket<'info>>,
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
        instructions::create_market::handler(
            ctx,
            market_id,
            title,
            outcomes,
            oracle,
            b,
            duration,
            fee_percent,
            fee_recipient,
            initial_funds,
        )
    }

    pub fn buy_shares(ctx: Context<BuyShares>, outcome_index: u64, num_shares: u64) -> Result<()> {
        instructions::buy_shares::handler(ctx, outcome_index, num_shares)
    }

    pub fn sell_shares(
        ctx: Context<SellShares>,
        outcome_index: u64,
        num_shares: u64,
    ) -> Result<()> {
        instructions::sell_shares::handler(ctx, outcome_index, num_shares)
    }

    pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
        instructions::close_market::handler(ctx)
    }

    pub fn set_outcome(ctx: Context<SetOutcome>, winning_outcome: u64) -> Result<()> {
        instructions::set_outcome::handler(ctx, winning_outcome)
    }
    pub fn claim_payout(ctx: Context<ClaimPayout>) -> Result<()> {
        instructions::claim_payout::handler(ctx)
    }
}

/*

    pub fn set_outcome(ctx: Context<SetOutcome>, outcome: u8) -> Result<()> {

    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {

    }
*/
