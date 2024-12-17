Implement calcualte cost/fee helper fucntions
Initial funds transfer to market while cretaing market=b ln(n)

Somewhere outcome mint is being initaizled with random mint authority other than market, due to which InalidMintAuth check fails in buyshares, if buyer_share_accout is decalred in BuyShares context with init_if_needed, this shouldn't happen, with it is happening
Should we perform a check in create_market.rs for outcome_mint initailziation and if exists then use that initialziation?
Are tests configured correctly beforeAll/beforeEach due to which same outcome mint is being initailzed multiple times
ideally buyer_sahre_account shoudl be init_if_needed but due to this problem of outcome_mint initailzatin beforehand we are keeping it mut 

#[account(
       mut,
        //  init_if_needed,
        //  payer = buyer,
        //  associated_token::mint = outcome_mint,
        //  associated_token::authority = buyer
     )]
    pub buyer_share_account: Account<'info, TokenAccount>,

when I use init if needed the authority of outcome changes, otherwise it is same as market as iniatlzed in create_market.rs
