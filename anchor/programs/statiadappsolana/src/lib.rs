#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");

#[program]
pub mod statiadappsolana {
    use super::*;

  pub fn close(_ctx: Context<CloseStatiadappsolana>) -> Result<()> {
    Ok(())
  }

  pub fn decrement(ctx: Context<Update>) -> Result<()> {
    ctx.accounts.statiadappsolana.count = ctx.accounts.statiadappsolana.count.checked_sub(1).unwrap();
    Ok(())
  }

  pub fn increment(ctx: Context<Update>) -> Result<()> {
    ctx.accounts.statiadappsolana.count = ctx.accounts.statiadappsolana.count.checked_add(1).unwrap();
    Ok(())
  }

  pub fn initialize(_ctx: Context<InitializeStatiadappsolana>) -> Result<()> {
    Ok(())
  }

  pub fn set(ctx: Context<Update>, value: u8) -> Result<()> {
    ctx.accounts.statiadappsolana.count = value.clone();
    Ok(())
  }
}

#[derive(Accounts)]
pub struct InitializeStatiadappsolana<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  #[account(
  init,
  space = 8 + Statiadappsolana::INIT_SPACE,
  payer = payer
  )]
  pub statiadappsolana: Account<'info, Statiadappsolana>,
  pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct CloseStatiadappsolana<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  #[account(
  mut,
  close = payer, // close account and return lamports to payer
  )]
  pub statiadappsolana: Account<'info, Statiadappsolana>,
}

#[derive(Accounts)]
pub struct Update<'info> {
  #[account(mut)]
  pub statiadappsolana: Account<'info, Statiadappsolana>,
}

#[account]
#[derive(InitSpace)]
pub struct Statiadappsolana {
  count: u8,
}
