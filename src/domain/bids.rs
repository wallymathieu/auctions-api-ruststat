
// src/domain/bids.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::money::Amount;
use super::core::{AuctionId, User};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bid {
    pub for_auction: AuctionId,
    pub bidder: User,
    pub at: DateTime<Utc>,
    pub bid_amount: Amount,
}