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

    // Initialize market metadata
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
    market.outcomes = vec![];

    // Loop through outcomes to create mint accounts
    for (i, outcome_name) in outcomes.iter().enumerate() {
        // Derive a unique seed and public key for the outcome mint
        let seed = format!("outcome_mint{}", i);
        let outcome_mint_key = Pubkey::create_with_seed(
            &ctx.accounts.user.key(),
            &seed,
            &spl_token::ID,
        ).map_err(|_| anchor_lang::error!(CustomError::InvalidMintKey))?;

        // Create the mint account using the derived key
        solana_program::program::invoke(
            &solana_program::system_instruction::create_account_with_seed(
                &ctx.accounts.user.key(),
                &outcome_mint_key,
                &ctx.accounts.user.key(),
                &seed,
                Rent::get()?.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::ID,
            ),
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Initialize the mint
        let outcome_mint_account = AccountInfo::new(
            &outcome_mint_key,
            false,
            true,
            &mut [].to_vec(),
            &ctx.accounts.user.key(),
            false,
            ctx.accounts.token_program.key(),
        );

        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                InitializeMint {
                    mint: outcome_mint_account.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            SHARES_DECIMALS.try_into().unwrap(),
            &market.key(),
            None,
        )?;

        // Add the outcome to the market
        market.outcomes.push(Outcome {
            name: outcome_name.clone(),
            total_shares: 0,
            mint: outcome_mint_key,
        });
    }

    Ok(())
}
