
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::money::Amount;
use super::core::{AuctionId, User};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bid {
    pub for_auction: AuctionId,
    pub bidder: User,
    #[serde(with="time::serde::rfc3339")]
    pub at: OffsetDateTime,
    pub bid_amount: Amount,
}