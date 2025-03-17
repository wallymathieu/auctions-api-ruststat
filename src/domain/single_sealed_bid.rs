// src/domain/single_sealed_bid.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use super::bids::Bid;
use super::core::{AuctionId, Errors, UserId};
use super::states::State;
use crate::money::Amount;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Options {
    /// Sealed first-price auction
    /// In this type of auction all bidders simultaneously submit sealed bids so that no bidder knows the bid of any
    /// other participant. The highest bidder pays the price they submitted.
    /// This type of auction is distinct from the English auction, in that bidders can only submit one bid each.
    Blind,
    
    /// Also known as a sealed-bid second-price auction.
    /// This is identical to the sealed first-price auction except that the winning bidder pays the second-highest bid
    /// rather than his or her own.
    Vickrey,
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Options::Blind => write!(f, "Blind"),
            Options::Vickrey => write!(f, "Vickrey"),
        }
    }
}

impl FromStr for Options {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Blind" => Ok(Options::Blind),
            "Vickrey" => Ok(Options::Vickrey),
            _ => Err(format!("Unknown SingleSealedBid option: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SingleSealedBidState {
    AcceptingBids {
        bids: HashMap<UserId, Bid>,
        expiry: DateTime<Utc>,
        options: Options,
    },
    DisclosingBids {
        bids: Vec<Bid>,
        expiry: DateTime<Utc>,
        options: Options,
    },
}

pub fn empty_state(expiry: DateTime<Utc>, options: Options) -> SingleSealedBidState {
    SingleSealedBidState::AcceptingBids {
        bids: HashMap::new(),
        expiry,
        options,
    }
}

impl State for SingleSealedBidState{

    fn inc(&self, now: DateTime<Utc>) -> Self {
        match self {
            SingleSealedBidState::AcceptingBids { bids, expiry, options } => {
                if now >= *expiry {
                    // Sort bids by amount (highest first)
                    let mut sorted_bids = bids.values().cloned().collect::<Vec<_>>();
                    sorted_bids.sort_by(|a, b| b.bid_amount.cmp(&a.bid_amount));
                    
                    SingleSealedBidState::DisclosingBids {
                        bids: sorted_bids,
                        expiry: *expiry,
                        options: options.clone(),
                    }
                } else {
                    self.clone()
                }
            },
            SingleSealedBidState::DisclosingBids { .. } => self.clone(),
        }
    }

    fn add_bid(&self, bid: Bid) -> (Self, Result<(), Errors>) {
        let now = bid.at;
        let auction_id = bid.for_auction;
        let user = bid.bidder.user_id().clone();
        
        let next = self.inc(now);
        
        match &next {
            SingleSealedBidState::AcceptingBids { bids, expiry, options } => {
                if bids.contains_key(&user) {
                    return (next, Err(Errors::AlreadyPlacedBid));
                }
                
                let mut new_bids = bids.clone();
                new_bids.insert(user, bid);
                
                (
                    SingleSealedBidState::AcceptingBids {
                        bids: new_bids,
                        expiry: *expiry,
                        options: options.clone(),
                    },
                    Ok(())
                )
            },
            SingleSealedBidState::DisclosingBids { .. } => {
                (next, Err(Errors::AuctionHasEnded(auction_id)))
            }
        }
    }

    fn get_bids(&self) -> Vec<Bid> {
        match self {
            SingleSealedBidState::DisclosingBids { bids, .. } => bids.clone(),
            _ => Vec::new(),
        }
    }

    fn try_get_amount_and_winner(&self) -> Option<(Amount, UserId)> {
        match self {
            SingleSealedBidState::AcceptingBids { .. } => None,
            SingleSealedBidState::DisclosingBids { bids, options, .. } => {
                if bids.is_empty() {
                    return None;
                }
                
                match options {
                    Options::Vickrey => {
                        if bids.len() == 1 {
                            // Only one bid, winner pays their own bid
                            Some((bids[0].bid_amount, bids[0].bidder.user_id().clone()))
                        } else {
                            // Winner pays the second highest bid
                            Some((bids[1].bid_amount, bids[0].bidder.user_id().clone()))
                        }
                    },
                    Options::Blind => {
                        // Winner pays their own bid
                        Some((bids[0].bid_amount, bids[0].bidder.user_id().clone()))
                    }
                }
            }
        }
    }

    fn has_ended(&self) -> bool {
        match self {
            SingleSealedBidState::AcceptingBids { .. } => false,
            SingleSealedBidState::DisclosingBids { .. } => true,
        }
    }

}


