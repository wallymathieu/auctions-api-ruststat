
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::money::Amount;
use super::core::{AuctionId, User};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bid {
    #[serde(rename = "auction")]
    pub for_auction: AuctionId,
    #[serde(rename = "user")]
    pub bidder: User,
    #[serde(with="time::serde::rfc3339")]
    pub at: OffsetDateTime,
    #[serde(rename = "amount")]
    pub bid_amount: Amount,
}