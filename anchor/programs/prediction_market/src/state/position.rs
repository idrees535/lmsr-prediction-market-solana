
use anchor_lang::prelude::*;

#[account]
pub struct Position {
    pub market: Pubkey,
    pub user: Pubkey,
    pub outcome_index: u64,
    pub num_shares: u64,
}
