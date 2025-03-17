// src/domain/mod.rs
pub mod auctions;
pub mod bids;
pub mod commands;
pub mod core;
pub mod states;
pub mod timed_ascending;
pub mod single_sealed_bid;

use std::collections::HashMap;
use thiserror::Error;

pub use self::auctions::*;
pub use self::bids::*;
pub use self::commands::*;
pub use self::core::*;
pub use self::states::*;

pub type Repository = HashMap<AuctionId, (Auction, AuctionState)>;

pub fn auctions(repository: &Repository) -> Vec<Auction> {
    repository.values().map(|(auction, _)| auction.clone()).collect()
}

#[derive(Debug, Error)]
pub enum HandleError {
    #[error("Auction error: {0}")]
    AuctionError(#[from] Errors),
}

pub fn handle(command: Command, mut repository: Repository) -> Result<(CommandSuccess, Repository), HandleError> {
    match command {
        Command::AddAuction { timestamp, auction } => {
            let auction_id = auction.auction_id;
            if !repository.contains_key(&auction_id) {
                let empty = empty_state(&auction);
                repository.insert(auction_id, (auction.clone(), empty));
                
                Ok((CommandSuccess::AuctionAdded { timestamp:timestamp, auction }, repository))
            } else {
                Err(HandleError::from(Errors::AuctionAlreadyExists(auction_id)))
            }
        }
        
        Command::PlaceBid { timestamp, bid } => {
            let auction_id = bid.for_auction;
            match repository.get(&auction_id) {
                Some((auction, state)) => {
                    validate_bid(&bid, auction)?;
                    
                    let (next_auction_state, bid_result) = add_bid(bid.clone(), state.clone());
                    bid_result?;
                    
                    repository.insert(auction_id, (auction.clone(), next_auction_state));
                    Ok((CommandSuccess::BidAccepted { timestamp, bid }, repository))
                }
                None => Err(HandleError::from(Errors::UnknownAuction(auction_id))),
            }
        }
    }
}