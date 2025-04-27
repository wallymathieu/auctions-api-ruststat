// src/domain/states.rs
use time::OffsetDateTime;
use crate::money::AmountValue;
use super::bids::Bid;
use super::core::{Errors, UserId};

pub trait State {
    fn inc(&self, now: OffsetDateTime) -> Self where Self: Sized;
    fn add_bid(&self, bid: Bid) -> (Self, Result<(), Errors>) where Self: Sized;
    fn get_bids(&self) -> Vec<Bid>;
    fn try_get_amount_and_winner(&self) -> Option<(AmountValue, UserId)>;
    fn has_ended(&self) -> bool;
}
