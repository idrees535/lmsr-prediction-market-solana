use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, InitializeMint};
use solana_program::pubkey::Pubkey;

pub fn create_mint<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateMint<'info>>,
    decimals: u8,
) -> Result<Pubkey> {
    let cpi_accounts = InitializeMint {
        mint: ctx.accounts.mint.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::initialize_mint(cpi_ctx, decimals, &ctx.accounts.authority.key(), None)?;

    Ok(ctx.accounts.mint.key())
}

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(init, payer = authority, space = Mint::LEN)]
    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
