use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use std::sync::{Arc, Mutex};

use crate::domain::{Auction, AuctionId, AuctionType, Repository, User};
use crate::money::{Currency, Amount, AmountValue};
use crate::domain::timed_ascending;

pub type AppState = Arc<Mutex<Repository>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BidRequest {
    pub amount: AmountValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddAuctionRequest {
    pub id: AuctionId,
    #[serde(with="time::serde::rfc3339", rename = "startsAt")]
    pub starts_at: OffsetDateTime,
    pub title: String,
    #[serde(with="time::serde::rfc3339", rename = "endsAt")]
    pub ends_at: OffsetDateTime,
    pub currency: Option<Currency>,
    pub typ: Option<AuctionType>,
}

impl AddAuctionRequest {
    pub fn to_auction(&self, seller: User) -> Auction {
        let currency = self.currency.unwrap_or(Currency::VAC);
        let typ = self.typ.clone().unwrap_or_else(|| {
            AuctionType::TimedAscending(timed_ascending::Options::default_options())
        });
        
        Auction {
            auction_id: self.id,
            starts_at: self.starts_at,
            title: self.title.clone(),
            expiry: self.ends_at,
            seller,
            typ,
            auction_currency: currency,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuctionItem {
    pub id: AuctionId,
    #[serde(with="time::serde::rfc3339", rename = "startsAt")]
    pub starts_at: OffsetDateTime,
    pub title: String,
    #[serde(with="time::serde::rfc3339")]
    pub expiry: OffsetDateTime,
    pub currency: Currency,
}

impl From<&Auction> for AuctionItem {
    fn from(auction: &Auction) -> Self {
        AuctionItem {
            id: auction.auction_id,
            starts_at: auction.starts_at,
            title: auction.title.clone(),
            expiry: auction.expiry,
            currency: auction.auction_currency,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuctionBid {
    pub amount: AmountValue,
    pub bidder: User,
}

#[derive(Debug, Serialize)]
pub struct AuctionDetail {
    // Base auction fields
    pub id: AuctionId,
    #[serde(with="time::serde::rfc3339", rename = "startsAt")]
    pub starts_at: OffsetDateTime,
    pub title: String,
    #[serde(with="time::serde::rfc3339")]
    pub expiry: OffsetDateTime,
    pub currency: Currency,
    
    // Additional detail fields
    pub bids: Vec<AuctionBid>,
    pub winner: Option<String>,
    #[serde(rename = "winnerPrice")]
    pub winner_price: Option<Amount>,
}