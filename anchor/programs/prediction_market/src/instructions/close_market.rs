use anchor_lang::prelude::*;
use crate::state::market::Market;
use crate::error::CustomError;

pub fn handler(ctx: Context<CloseMarket>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let current_time = Clock::get()?.unix_timestamp;

    // Ensure the market is not already closed
    require!(!market.market_closed, CustomError::MarketAlreadyClosed);

    // Check if the market's end time has passed
    require!(
        current_time >= market.end_timestamp,
        CustomError::MarketNotExpired
    );

    // Mark the market as closed
    market.market_closed = true;
    msg!("Market {} has been closed successfully", market.market_id);

    Ok(())
}

#[derive(Accounts)]
pub struct CloseMarket<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>, // Ensure the oracle is correct
    
    #[account(
        signer, // Ensure this account is signed
        //address = market.oracle // Ensure this is the oracle account defined in the market
    )]
    pub oracle: Signer<'info>, // The oracle signs the transaction 
    pub system_program: Program<'info, System>,   
}
