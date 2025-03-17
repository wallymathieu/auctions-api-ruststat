// src/domain/auctions.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use core::fmt;
use std::str::FromStr;
use crate::money::Currency;
use super::bids::Bid;
use super::core::{AuctionId, Errors, User};
use super::single_sealed_bid::Options as SBOptions;
use super::timed_ascending::Options as TAOptions;
use super::states::State;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuctionType {
    TimedAscending(TAOptions),
    SingleSealedBid(SBOptions),
}

impl Serialize for AuctionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for AuctionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let text = String::deserialize(deserializer)?;
        AuctionType::from_str(&text).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for AuctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuctionType::TimedAscending(opts) => write!(f, "{}", opts),
            AuctionType::SingleSealedBid(opts) => write!(f, "{}", opts),
        }
    }
}

impl FromStr for AuctionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(opts) = TAOptions::from_str(s) {
            return Ok(AuctionType::TimedAscending(opts));
        }
        
        if let Ok(opts) = SBOptions::from_str(s) {
            return Ok(AuctionType::SingleSealedBid(opts));
        }
        
        Err(format!("Unknown auction type: {}", s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Auction {
    #[serde(rename = "id")]
    pub auction_id: AuctionId,
    #[serde(rename = "startsAt")]
    pub starts_at: DateTime<Utc>,
    pub title: String,
    pub expiry: DateTime<Utc>,
    #[serde(rename = "user")]
    pub seller: User,
    #[serde(rename = "type")]
    pub typ: AuctionType,
    #[serde(rename = "currency")]
    pub auction_currency: Currency,
}

pub fn validate_bid(bid: &Bid, auction: &Auction) -> Result<(), Errors> {
    if bid.bidder.user_id() == auction.seller.user_id() {
        return Err(Errors::SellerCannotPlaceBids((
            bid.bidder.user_id().clone(), 
            auction.auction_id
        )));
    }
    
    if bid.bid_amount.currency() != auction.auction_currency {
        return Err(Errors::CurrencyConversion(auction.auction_currency));
    }
    
    Ok(())
}

#[derive(Debug, Clone)]
pub enum AuctionState {
    SingleSealedBid(super::single_sealed_bid::SingleSealedBidState),
    TimedAscending(super::timed_ascending::TimedAscendingState),
}

pub fn empty_state(auction: &Auction) -> AuctionState {
    match &auction.typ {
        AuctionType::SingleSealedBid(opt) => {
            AuctionState::SingleSealedBid(
                super::single_sealed_bid::empty_state(auction.expiry, opt.clone())
            )
        },
        AuctionType::TimedAscending(opt) => {
            AuctionState::TimedAscending(
                super::timed_ascending::empty_state(auction.starts_at, auction.expiry, opt.clone())
            )
        }
    }
}

impl State for AuctionState{
    fn inc(&self, now: DateTime<Utc>) -> Self where Self: Sized {
        match self {
            AuctionState::SingleSealedBid(state) => {
                AuctionState::SingleSealedBid(state.inc(now))
            },
            AuctionState::TimedAscending(state) => {
                AuctionState::TimedAscending(state.inc(now))
            }
        }
    }

    fn add_bid(&self, bid: Bid) -> (Self, Result<(), Errors>) where Self: Sized {
        match self {
            AuctionState::SingleSealedBid(state) => {
                let (new_state, result) = state.add_bid(bid);
                (AuctionState::SingleSealedBid(new_state), result)
            },
            AuctionState::TimedAscending(state) => {
                let (new_state, result) = state.add_bid(bid);
                (AuctionState::TimedAscending(new_state), result)
            }
        }
    }

    fn get_bids(&self) -> Vec<Bid> {
        match self {
            AuctionState::SingleSealedBid(state) => state.get_bids(),
            AuctionState::TimedAscending(state) => state.get_bids()
        }
    }

    fn try_get_amount_and_winner(&self) -> Option<(crate::Amount, super::UserId)> {
        match self {
            AuctionState::SingleSealedBid(state) => state.try_get_amount_and_winner(),
            AuctionState::TimedAscending(state) => state.try_get_amount_and_winner()
        }
    }

    fn has_ended(&self) -> bool {
        match self {
            AuctionState::SingleSealedBid(state) => state.has_ended(),
            AuctionState::TimedAscending(state) => state.has_ended()
        }
    }
}