// src/domain/mod.rs
pub mod auctions;
pub mod bids;
pub mod commands;
pub mod core;
pub mod states;
pub mod timed_ascending;
pub mod single_sealed_bid;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use thiserror::Error;

pub use self::auctions::*;
pub use self::bids::*;
pub use self::commands::*;
pub use self::core::*;
pub use self::states::*;

pub type Repository = HashMap<AuctionId, (Auction, AuctionState)>;

#[derive(Debug, Error)]
pub enum HandleError {
    #[error("Auction error: {0}")]
    AuctionError(#[from] Errors),
}

pub fn handle(command: Command, repository: &mut Repository) -> Result<Event, HandleError> {
    match command {
        Command::AddAuction { timestamp, auction } => {
            let auction_id = auction.auction_id;
            match repository.entry(auction_id) {
                Entry::Vacant(entry) => {
                    let empty = empty_state(&auction);
                    entry.insert((auction.clone(), empty));

                    Ok(Event::AuctionAdded { timestamp, auction })
                }
                Entry::Occupied(_) => {
                    Err(HandleError::from(Errors::AuctionAlreadyExists(auction_id)))
                }
            }
        }

        Command::PlaceBid { timestamp, bid } => {
            let auction_id = bid.for_auction;
            match repository.get_mut(&auction_id) {
                Some((auction, state)) => {
                    validate_bid(&bid, auction)?;

                    let (next_auction_state, bid_result) = state.add_bid(bid.clone());
                    bid_result?;

                    *state = next_auction_state;
                    Ok(Event::BidAccepted { timestamp, bid })
                }
                None => Err(HandleError::from(Errors::UnknownAuction(auction_id))),
            }
        }
    }
}
