use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use crate::money::{Amount, Currency};
use super::bids::Bid;
use super::core::{AuctionId, Errors, UserId};
use super::states::State;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Options {
    /// The seller has set a minimum sale price in advance (the 'reserve' price)
    /// and the final bid does not reach that price the item remains unsold.
    /// If the reserve price is 0, that is the equivalent of not setting it.
    pub reserve_price: Amount,
    
    /// Sometimes the auctioneer sets a minimum amount by which the next bid must exceed the current highest bid.
    /// Having min raise equal to 0 is the equivalent of not setting it.
    pub min_raise: Amount,
    
    /// If no competing bidder challenges the standing bid within a given time frame,
    /// the standing bid becomes the winner, and the item is sold to the highest bidder
    /// at a price equal to his or her bid.
    pub time_frame: Duration,
}

impl Options {
    pub fn default_options(currency: Currency) -> Self {
        Options {
            reserve_price: Amount::new(currency, 0),
            min_raise: Amount::new(currency, 0),
            time_frame: Duration::seconds(0),
        }
    }
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "English|{}|{}|{}",
            self.reserve_price,
            self.min_raise,
            self.time_frame.num_seconds()
        )
    }
}

impl FromStr for Options {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 4 || parts[0] != "English" {
            return Err(format!("Invalid TimedAscending options format: {}", s));
        }
        
        let reserve_price = parts[1].parse::<Amount>()
            .map_err(|e| format!("Invalid reserve price: {}", e))?;
            
        let min_raise = parts[2].parse::<Amount>()
            .map_err(|e| format!("Invalid min raise: {}", e))?;
            
        let time_frame_seconds = parts[3].parse::<i64>()
            .map_err(|_| format!("Invalid time frame: {}", parts[3]))?;
            
        Ok(Options {
            reserve_price,
            min_raise,
            time_frame: Duration::seconds(time_frame_seconds),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimedAscendingState {
    AwaitingStart {
        start: DateTime<Utc>,
        starting_expiry: DateTime<Utc>,
        options: Options,
    },
    OnGoing {
        bids: Vec<Bid>,
        next_expiry: DateTime<Utc>,
        options: Options,
    },
    HasEnded {
        bids: Vec<Bid>,
        expiry: DateTime<Utc>,
        options: Options,
    },
}

pub fn empty_state(start: DateTime<Utc>, starting_expiry: DateTime<Utc>, options: Options) -> TimedAscendingState {
    TimedAscendingState::AwaitingStart {
        start,
        starting_expiry,
        options,
    }
}

impl State for TimedAscendingState {
    fn inc(&self, now: DateTime<Utc>) -> Self {
        match self {
            TimedAscendingState::AwaitingStart { start, starting_expiry, options } => {
                if now > *start {
                    if now < *starting_expiry {
                        // AwaitingStart -> OnGoing
                        TimedAscendingState::OnGoing {
                            bids: Vec::new(),
                            next_expiry: *starting_expiry,
                            options: options.clone(),
                        }
                    } else {
                        // AwaitingStart -> HasEnded
                        TimedAscendingState::HasEnded {
                            bids: Vec::new(),
                            expiry: *starting_expiry,
                            options: options.clone(),
                        }
                    }
                } else {
                    // AwaitingStart -> AwaitingStart
                    self.clone()
                }
            },
            TimedAscendingState::OnGoing { bids, next_expiry, options } => {
                if now < *next_expiry {
                    // OnGoing -> OnGoing
                    self.clone()
                } else {
                    // OnGoing -> HasEnded
                    TimedAscendingState::HasEnded {
                        bids: bids.clone(),
                        expiry: *next_expiry,
                        options: options.clone(),
                    }
                }
            },
            TimedAscendingState::HasEnded { .. } => {
                // HasEnded -> HasEnded
                self.clone()
            }
        }
    }

    fn add_bid(&self, bid: Bid) -> (Self, Result<(), Errors>) {
        let now = bid.at;
        let auction_id = bid.for_auction;
        let bid_amount = bid.bid_amount;
        
        let next = self.inc(now);
        
        match &next {
            TimedAscendingState::AwaitingStart { .. } => {
                (next, Err(Errors::AuctionHasNotStarted(auction_id)))
            },
            TimedAscendingState::OnGoing { bids, next_expiry, options } => {
                let mut new_bids = bids.clone();
                let new_expiry = std::cmp::max(
                    *next_expiry,
                    now + options.time_frame
                );
                
                if bids.is_empty() {
                    // First bid is always accepted
                    new_bids.insert(0, bid);
                    (
                        TimedAscendingState::OnGoing {
                            bids: new_bids,
                            next_expiry: new_expiry,
                            options: options.clone(),
                        },
                        Ok(())
                    )
                } else {
                    // Check if the bid is high enough
                    let highest_bid = &bids[0];
                    let highest_amount = highest_bid.bid_amount;
                    let min_raise = options.min_raise;
                    
                    // You cannot bid lower than the current bid + minimum raise
                    if bid_amount.value() >= (highest_amount.value() + min_raise.value()) {
                        new_bids.insert(0, bid);
                        (
                            TimedAscendingState::OnGoing {
                                bids: new_bids,
                                next_expiry: new_expiry,
                                options: options.clone(),
                            },
                            Ok(())
                        )
                    } else {
                        (next, Err(Errors::MustPlaceBidOverHighestBid(highest_amount)))
                    }
                }
            },
            TimedAscendingState::HasEnded { .. } => {
                (next, Err(Errors::AuctionHasEnded(auction_id)))
            }
        }
    }

    fn get_bids(&self) -> Vec<Bid> {
        match self {
            TimedAscendingState::AwaitingStart { .. } => Vec::new(),
            TimedAscendingState::OnGoing { bids, .. } => bids.clone(),
            TimedAscendingState::HasEnded { bids, .. } => bids.clone(),
        }
    }

    fn try_get_amount_and_winner(&self) -> Option<(Amount, UserId)> {
        match self {
            TimedAscendingState::HasEnded { bids, options, .. } => {
                if let Some(bid) = bids.first() {
                    if options.reserve_price.value() < bid.bid_amount.value() {
                        return Some((bid.bid_amount, bid.bidder.user_id().clone()));
                    }
                }
                None
            },
            _ => None,
        }
    }

    fn has_ended(&self) -> bool {
        match self {
            TimedAscendingState::HasEnded { .. } => true,
            _ => false,
        }
    }
}

