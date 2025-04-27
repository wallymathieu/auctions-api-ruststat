// src/domain/core.rs
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub type UserId = String;
pub type AuctionId = i64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum User {
    BuyerOrSeller {
        user_id: UserId,
        name: String,
    },
    Support {
        user_id: UserId,
    },
}

impl User {
    pub fn user_id(&self) -> &UserId {
        match self {
            User::BuyerOrSeller { user_id, .. } => user_id,
            User::Support { user_id } => user_id,
        }
    }
}
impl<'de> Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        let parts: Vec<&str> = text.split('|').collect();
        
        match parts.as_slice() {
            ["BuyerOrSeller", user_id, name] => {
                Ok(User::BuyerOrSeller {
                    user_id: user_id.to_string(),
                    name: name.to_string(),
                })
            },
            ["Support", user_id] => {
                Ok(User::Support {
                    user_id: user_id.to_string(),
                })
            },
            _ => {
                Err(serde::de::Error::custom(
                    format!("parsing User failed, could not interpret values: {:?}", parts)
                ))
            }
        }
    }
}
impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            User::BuyerOrSeller { user_id, name } => write!(f, "BuyerOrSeller|{}|{}", user_id, name),
            User::Support { user_id } => write!(f, "Support|{}", user_id),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Errors {
    #[error("Unknown auction: {0}")]
    UnknownAuction(AuctionId),
    
    #[error("Auction already exists: {0}")]
    AuctionAlreadyExists(AuctionId),
    
    #[error("Auction has ended: {0}")]
    AuctionHasEnded(AuctionId),
    
    #[error("Auction has not started: {0}")]
    AuctionHasNotStarted(AuctionId),
    
    #[error("Seller cannot place bids: {0:?}")]
    SellerCannotPlaceBids((UserId, AuctionId)),
    
    #[error("Invalid user data: {0}")]
    InvalidUserData(String),
    
    #[error("Must place bid over highest bid: {0:?}")]
    MustPlaceBidOverHighestBid(i64),
    
    #[error("Already placed bid")]
    AlreadyPlacedBid,
}