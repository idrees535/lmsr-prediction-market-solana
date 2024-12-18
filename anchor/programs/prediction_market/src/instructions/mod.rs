// In src/instructions/mod.rs
pub mod create_market;
pub mod buy_shares;
pub mod sell_shares;
pub mod close_market;
pub mod set_outcome;
pub mod claim_payout;
pub mod withdraw_fees;

pub use create_market::CreateMarket;
pub use create_market::*;
pub use buy_shares::BuyShares;
pub use buy_shares::*;
pub use sell_shares::SellShares;
pub use sell_shares::*;
pub use close_market::CloseMarket;
pub use close_market::*;
pub use set_outcome::SetOutcome;
pub use set_outcome::*;
pub use claim_payout::ClaimPayout;
pub use claim_payout::*;
pub use withdraw_fees::WithdrawFees;
pub use withdraw_fees::*;

