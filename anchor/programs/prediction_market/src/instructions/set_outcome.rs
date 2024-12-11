use anchor_lang::prelude::*;
use crate::state::market::Market;
use crate::error::CustomError;

pub fn handler(
    ctx: Context<SetOutcome>,
    winning_outcome: u64,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Validations
    require!(market.market_closed, CustomError::MarketNotClosed);
    require!(!market.market_settled, CustomError::MarketAlreadySettled);
    require!(winning_outcome < market.outcomes.len() as u64, CustomError::InvalidOutcome);

    market.winning_outcome = winning_outcome;
    market.market_settled = true;

    msg!("Outcome Set: {}", winning_outcome);

    Ok(())
}

#[derive(Accounts)]
pub struct SetOutcome<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    pub oracle: Signer<'info>, // Oracle must be the signer
}
