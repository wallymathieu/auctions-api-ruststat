use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use super::auctions::Auction;
use super::bids::Bid;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum Command {
    #[serde(rename = "AddAuction")]
    AddAuction {
        #[serde(with="time::serde::rfc3339", rename = "at")]
        timestamp: OffsetDateTime,
        auction: Auction,
    },
    
    #[serde(rename = "PlaceBid")]
    PlaceBid {
        #[serde(with="time::serde::rfc3339", rename = "at")]
        timestamp: OffsetDateTime,
        bid: Bid,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum CommandSuccess {
    #[serde(rename = "AuctionAdded")]
    AuctionAdded {
        #[serde(with="time::serde::rfc3339", rename = "at")]
        timestamp: OffsetDateTime,
        auction: Auction,
    },
    
    #[serde(rename = "BidAccepted")]
    BidAccepted {
        #[serde(with="time::serde::rfc3339", rename = "at")]
        timestamp: OffsetDateTime,
        bid: Bid,
    },
}