use anchor_lang::prelude::*;
use crate::state::market::Market;
use crate::state::outcome::Outcome;

#[derive(Accounts)]
#[instruction(
    market_id: u64, 
    title: String, 
    outcomes: Vec<String>, 
    oracle: Pubkey, 
    b: u64, 
    duration: i64, 
    fee_percent: u64, 
    fee_recipient: Pubkey, 
    token_mint: Pubkey, 
    initial_funds: u64
)]
pub struct CreateMarket<'info> {
    #[account(
        init,
        payer = creator,
        seeds = [b"market", &market_id.to_le_bytes()[..]],
        bump,
        // Space is an approximation, you can refine later
        space = 8 + 8 + 260 + 32 + 8 + 8 + 1 + 1 + 8 + 8 + 8 + 8 + 32 + 8 + 32 + 760
    )]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateMarket>,
    market_id: u64,
    title: String,
    outcomes: Vec<String>,
    oracle: Pubkey,
    b: u64,
    duration: i64,
    fee_percent: u64,
    fee_recipient: Pubkey,
    token_mint: Pubkey,
    initial_funds: u64,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let clock = Clock::get()?;

    market.market_id = market_id;
    market.title = title;
    market.oracle = oracle;
    market.b = b;
    market.market_end_ts = clock.unix_timestamp + duration;
    market.market_closed = false;
    market.market_settled = false;
    market.winning_outcome = 0;
    market.market_maker_funds = initial_funds;
    market.initial_funds = initial_funds;
    market.fee_percent = fee_percent;
    market.fee_recipient = fee_recipient;
    market.collected_fees = 0;
    market.token_mint = token_mint;

    let mapped_outcomes = outcomes.into_iter().map(|name| Outcome {
        name,
        total_shares: 0,
    }).collect();

    market.outcomes = mapped_outcomes;

    Ok(())
}
