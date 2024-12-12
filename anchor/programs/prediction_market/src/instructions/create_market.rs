use anchor_lang::prelude::*;
use anchor_spl::token::{Token, spl_token};
//use anchor_spl::token_interface::Mint;
use anchor_spl::token::{Mint};
use crate::state::market::Market;
use crate::state::outcome::Outcome;
use crate::error::CustomError;
use solana_program::program_pack::Pack;

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
    initial_funds: u64,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let clock = Clock::get()?;

    require!(outcomes.len() > 0, CustomError::NoOutcomes);
    require!(b > 0, CustomError::InvalidB);
    require!(duration > 0, CustomError::InvalidDuration);
    // require!(
    //     *ctx.accounts.base_token_mint.owner == spl_token::ID,
    //     CustomError::InvalidOwner
    // );

    require!(
        ctx.accounts.base_token_mint.to_account_info().data_len() == spl_token::state::Mint::LEN,
        CustomError::InvalidMint
    );

    let mint = spl_token::state::Mint::unpack(
        &ctx.accounts.base_token_mint.to_account_info().data.borrow(),
    )?;
    msg!("Mint supply: {}", mint.supply);
    msg!("Market Base Token Mint: {}", market.base_token_mint);

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
    market.base_token_mint = ctx.accounts.base_token_mint.key();

    let mut outcomes_structs: Vec<Outcome> = Vec::with_capacity(outcomes.len());
    for outcome_name in outcomes.iter() {
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

#[derive(Accounts)]
#[instruction(market_id: u64, title: String, outcomes: Vec<String>, oracle: Pubkey, b: u64, duration: i64, fee_percent: u64, fee_recipient: Pubkey, initial_funds: u64)]
pub struct CreateMarket<'info> {
    #[account(
        init,
        seeds = [b"market", market_id.to_le_bytes().as_ref()],
        bump,
        payer = user,
        space = 8 + Market::INIT_SPACE,
    )]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub user: Signer<'info>,

    ///CHECK: The base_token_mint is provided by the user. Checked at runtime.
    //pub base_token_mint: UncheckedAccount<'info>,

    #[account(mint::token_program=token_program)]
    pub base_token_mint: Account<'info, Mint>,

    #[account(address = anchor_lang::solana_program::system_program::ID)]
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
