use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Outcome {
    pub name: String,
    pub total_shares: u64,
    pub mint: Pubkey,
}
