// src/domain/states.rs
use time::OffsetDateTime;
use crate::money::Amount;
use super::bids::Bid;
use super::core::{Errors, UserId};
use super::AuctionState;

pub trait State {
    fn inc(&self, now: OffsetDateTime) -> Self where Self: Sized;
    fn add_bid(&self, bid: Bid) -> (Self, Result<(), Errors>) where Self: Sized;
    fn get_bids(&self) -> Vec<Bid>;
    fn try_get_amount_and_winner(&self) -> Option<(Amount, UserId)>;
    fn has_ended(&self) -> bool;
}

// Helper functions to work with AuctionState
pub fn inc(state: &AuctionState, now: OffsetDateTime) -> AuctionState {
    State::inc(state, now)
}

pub fn add_bid(bid: Bid, state: AuctionState) -> (AuctionState, Result<(), Errors>) {
    State::add_bid(&state, bid)
}

pub fn get_bids(state: &AuctionState) -> Vec<Bid> {
    State::get_bids(state)
}

pub fn try_get_amount_and_winner(state: &AuctionState) -> Option<(Amount, UserId)> {
    State::try_get_amount_and_winner(state)
}

pub fn has_ended(state: &AuctionState) -> bool {
    State::has_ended(state)
}