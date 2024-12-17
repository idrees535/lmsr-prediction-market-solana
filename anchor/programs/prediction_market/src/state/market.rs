use anchor_lang::prelude::*;
use crate::state::outcome::Outcome;


#[account]
#[derive(InitSpace)]
pub struct Market {
    pub market_id: u64,
    #[max_len(50)]
    pub title: String,
    pub oracle: Pubkey,
    pub b: u64,
    pub fee_percent: u64,
    pub fee_recipient: Pubkey,
    #[max_len(10,50)]
    pub outcomes: Vec<Outcome>,
    pub end_timestamp: i64,       
    pub market_closed: bool,
    pub market_settled: bool,
    pub winning_outcome: u64,
    pub market_maker_funds: u64,
    pub initial_funds: u64,
    pub collected_fees: u64,
    pub base_token_mint: Pubkey,
    pub bump: u8,
    //pub outcome_mint: Pubkey, 
}
