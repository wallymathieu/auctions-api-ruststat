use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::auctions::Auction;
use super::bids::Bid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum Command {
    #[serde(rename = "AddAuction")]
    AddAuction {
        #[serde(rename = "at")]
        timestamp: DateTime<Utc>,
        auction: Auction,
    },
    
    #[serde(rename = "PlaceBid")]
    PlaceBid {
        #[serde(rename = "at")]
        timestamp: DateTime<Utc>,
        bid: Bid,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum CommandSuccess {
    #[serde(rename = "AuctionAdded")]
    AuctionAdded {
        #[serde(rename = "at")]
        timestamp: DateTime<Utc>,
        auction: Auction,
    },
    
    #[serde(rename = "BidAccepted")]
    BidAccepted {
        #[serde(rename = "at")]
        timestamp: DateTime<Utc>,
        bid: Bid,
    },
}