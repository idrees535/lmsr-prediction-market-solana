// In src/instructions/mod.rs
pub mod create_market;
pub mod buy_shares;
pub mod sell_shares;
pub mod set_outcome;
pub mod claim_payout;
pub mod withdraw_fees;

pub use create_market::CreateMarket;
pub use buy_shares::BuyShares;
pub use sell_shares::SellShares;
pub use set_outcome::SetOutcome;
pub use claim_payout::ClaimPayout;
pub use withdraw_fees::WithdrawFees;
