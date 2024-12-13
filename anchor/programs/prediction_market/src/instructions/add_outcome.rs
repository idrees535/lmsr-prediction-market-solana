use anchor_lang::prelude::*;
use anchor_spl::token::{self, spl_token, InitializeMint, Token};
//use anchor_spl::token_interface::Mint;
use anchor_spl::token::{Mint};
use crate::constants::SHARES_DECIMALS;
use crate::state::market::Market;
use crate::state::outcome::Outcome;
use crate::error::CustomError;
use solana_program::program_pack::Pack;

pub fn handler(
    ctx: Context<AddOutcome>,
    name: String,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Validate name length
    require!(name.len() <= 50, CustomError::OutcomeNameTooLong);

    // Add outcome to the market
    market.outcomes.push(Outcome {
        name,
        total_shares: 0,
        mint: ctx.accounts.outcome_mint.key(),
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct AddOutcome<'info> {
    #[account(
        mut,
        has_one = base_token_mint,
        seeds = [b"market", market.market_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub market: Account<'info, Market>,

    #[account(
        init,
        payer = user,
        seeds = [b"outcome", market.key().as_ref(), name.as_bytes()],
        bump,
        mint::authority = market,
        mint::decimals = 0,
    )]
    pub outcome_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub base_token_mint: Account<'info, Mint>,

    #[account(address = anchor_lang::solana_program::system_program::ID)]
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
