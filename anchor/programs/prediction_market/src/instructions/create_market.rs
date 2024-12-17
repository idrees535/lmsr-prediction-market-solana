use anchor_lang::prelude::*;
use anchor_spl::token::{self, spl_token, InitializeMint, Token};
//use anchor_spl::token_interface::Mint;
use crate::constants::SHARES_DECIMALS;
use crate::error::CustomError;
use crate::state::market::Market;
use crate::state::outcome::Outcome;
use anchor_spl::token::Mint;
use solana_program::program_pack::Pack;

pub fn handler<'info>(
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
    let market = &mut ctx.accounts.market;
    let base_token_mint_info = ctx.accounts.base_token_mint.to_account_info();
    let clock = Clock::get()?;

    require!(outcomes.len() > 0, CustomError::NoOutcomes);
    require!(b > 0, CustomError::InvalidB);
    require!(duration > 0, CustomError::InvalidDuration);

    require!(
        base_token_mint_info.data_len() == spl_token::state::Mint::LEN,
        CustomError::InvalidMint
    );

    let mint = spl_token::state::Mint::unpack(&base_token_mint_info.data.borrow())?;
    msg!("Mint supply: {}", mint.supply);
    msg!("Market Base Token Mint: {}", market.base_token_mint);

    let bump = ctx.bumps.market;
    market.bump = bump;

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
    //market.outcomes = vec![]; // Initialize outcomes

    // Dynamically process remaining accounts as outcome mints
    let remaining_accounts = &ctx.remaining_accounts;
    require!(
        remaining_accounts.len() == outcomes.len(),
        CustomError::InvalidAccounts
    );

    for (i, outcome_name) in outcomes.iter().enumerate() {
        let outcome_mint = &remaining_accounts[i].clone();
        msg!("Outcome Mint: {}", outcome_mint.key());
        require!(
            *outcome_mint.owner == spl_token::id(),
            CustomError::InvalidMint
        );

        // Check if the mint account is already initialized
        if spl_token::state::Mint::unpack(&outcome_mint.data.borrow()).is_ok() {
            msg!("Outcome Mint already initialized: {}", outcome_mint.key());
            // should we allow an already initialized mint? Don't think so
            // require!(
            //     spl_token::state::Mint::unpack(&outcome_mint.data.borrow()).is_err(),
            //     CustomError::MintAlreadyInitialized
            //);
        } else {
            msg!(
                "Outcome mints not Initalized, Initializing it now: {}",
                outcome_mint.key()
            );

            //Initialize the mint for the outcome
            token::initialize_mint(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    InitializeMint {
                        mint: outcome_mint.clone(),
                        rent: ctx.accounts.rent.to_account_info(),
                    },
                ),
                SHARES_DECIMALS.try_into().unwrap(), 
                &market.to_account_info().key(),
                None, // Freeze authority
            )?;
            // Print the mint authority after initialization
        //let mint_state = spl_token::state::Mint::unpack(&outcome_mint.data.borrow())?;
        // msg!(
        //     "Outcome Mint Initialized: {} with Mint Authority: {}",
        //     outcome_mint.key(),
        //     mint_state.mint_authority.unwrap()
        // );
        }

        // Add the outcome to the market
        market.outcomes.push(Outcome {
            name: outcome_name.clone(),
            total_shares: 0,
            mint: outcome_mint.key(),
        });
    }

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

    #[account(mint::token_program=token_program)]
    pub base_token_mint: Account<'info, Mint>,

    #[account(address = anchor_lang::solana_program::system_program::ID)]
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
