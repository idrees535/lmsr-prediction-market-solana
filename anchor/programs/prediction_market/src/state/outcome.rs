use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
#[derive(InitSpace)]
pub struct Outcome {
    #[max_len(50)]
    pub name: String,
    pub total_shares: u64,
    pub mint: Pubkey,
}
