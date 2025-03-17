use auction_site::domain::{
    AuctionType, Auction, User, Bid, Command, CommandSuccess,
    timed_ascending::Options as TAOptions,
    single_sealed_bid::Options as SBOptions,
};
use auction_site::money::{Amount, Currency};
use auction_site::persistence::json_file::{read_commands, write_commands};
use serde_json::{from_str, to_string};
use time::format_description::well_known::Rfc3339;
use time::macros::datetime;
use time::{Duration, OffsetDateTime};
use std::str::FromStr;
use std::fs;
use std::path::Path;

// Sample data for tests
fn sample_auction_id() -> i64 {
    1
}

fn sample_title() -> String {
    "auction".to_string()
}

fn sample_starts_at() -> OffsetDateTime {
    datetime!(2016-01-01 8:28 UTC)
}

fn sample_ends_at() -> OffsetDateTime {
    datetime!(2016-02-01 8:28 UTC)
}

fn sample_bid_time() -> OffsetDateTime {
    datetime!(2016-01-15 8:28 UTC)
}


fn sample_seller() -> User {
    User::BuyerOrSeller {
        user_id: "Sample_Seller".to_string(),
        name: "Seller".to_string(),
    }
}

fn buyer_1() -> User {
    User::BuyerOrSeller {
        user_id: "Buyer_1".to_string(),
        name: "Buyer 1".to_string(),
    }
}

fn sek(value: i64) -> Amount {
    Amount::new(Currency::SEK, value)
}

fn vac(value: i64) -> Amount {
    Amount::new(Currency::VAC, value)
}

fn sample_auction() -> Auction {
    Auction {
        auction_id: sample_auction_id(),
        title: sample_title(),
        starts_at: sample_starts_at(),
        expiry: sample_ends_at(),
        seller: sample_seller(),
        auction_currency: Currency::SEK,
        typ: AuctionType::SingleSealedBid(SBOptions::Vickrey),
    }
}

fn bid_1() -> Bid {
    Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_1(),
        at: sample_starts_at() + Duration::seconds(1),
        bid_amount: sek(10),
    }
}

#[test]
fn test_read_json_commands() {
    // Read sample commands from file
    let commands = read_commands("./tests/samples/sample-commands.jsonl");
    assert!(commands.is_ok());
    assert!(!commands.unwrap().is_empty());
}

#[test]
fn test_auction_type_serialization() {
    // TimedAscending auction type
    let vac0 = vac(0);
    let timed_ascending = AuctionType::TimedAscending(
        TAOptions {
            reserve_price: vac0,
            min_raise: vac0,
            time_frame: Duration::seconds(0),
        }
    );
    
    // Can serialize to JSON
    let serialized = to_string(&timed_ascending).unwrap();
    
    // Can deserialize from JSON
    let deserialized: AuctionType = from_str(&serialized).unwrap();
    
    // Types should match
    match deserialized {
        AuctionType::TimedAscending(opts) => {
            assert_eq!(opts.reserve_price, vac0);
            assert_eq!(opts.min_raise, vac0);
            assert_eq!(opts.time_frame, Duration::seconds(0));
        },
        _ => panic!("Expected TimedAscending type"),
    }
    
    // Also check direct string parsing
    let type_str = "English|VAC0|VAC0|0";
    let parsed = AuctionType::from_str(type_str).unwrap();
    
    match parsed {
        AuctionType::TimedAscending(opts) => {
            assert_eq!(opts.reserve_price, vac0);
            assert_eq!(opts.min_raise, vac0);
            assert_eq!(opts.time_frame, Duration::seconds(0));
        },
        _ => panic!("Expected TimedAscending type"),
    }
}

#[test]
fn test_amount_serialization() {
    let amount = vac(0);
    
    // Can parse amount string
    let parsed = Amount::from_str("VAC0").unwrap();
    assert_eq!(parsed, amount);
    
    // Can convert back to string
    assert_eq!(amount.to_string(), "VAC0");
    
    // Roundtrip through JSON
    let serialized = to_string(&amount).unwrap();
    let deserialized: Amount = from_str(&serialized).unwrap();
    assert_eq!(deserialized, amount);
}

#[test]
fn test_add_auction_command_serialization() {
    let auction = sample_auction();
    let add_auction = Command::AddAuction {
        timestamp: sample_starts_at(),
        auction: auction.clone(),
    };
    
    // Serialize to JSON
    let serialized = to_string(&add_auction).unwrap();
    
    // Verify it contains the expected data
    let json_value = serde_json::from_str::<serde_json::Value>(&serialized).unwrap();
    assert_eq!(json_value["$type"], "AddAuction");
    assert_eq!(json_value["at"], sample_starts_at().format(&Rfc3339).unwrap());
    
    // Deserialize back
    let deserialized: Command = from_str(&serialized).unwrap();
    
    // Verify it matches the original
    match deserialized {
        Command::AddAuction { timestamp, auction: deserialized_auction } => {
            assert_eq!(timestamp, sample_starts_at());
            assert_eq!(deserialized_auction, auction);
        },
        _ => panic!("Expected AddAuction command"),
    }
}

#[test]
fn test_place_bid_command_serialization() {
    let bid = bid_1();
    let place_bid = Command::PlaceBid {
        timestamp: sample_bid_time(),
        bid: bid.clone(),
    };
    
    // Serialize to JSON
    let serialized = to_string(&place_bid).unwrap();
    
    // Verify it contains the expected data
    let json_value = serde_json::from_str::<serde_json::Value>(&serialized).unwrap();
    assert_eq!(json_value["$type"], "PlaceBid");
    assert_eq!(json_value["at"], sample_bid_time().format(&Rfc3339).unwrap());
    
    // Deserialize back
    let deserialized: Command = from_str(&serialized).unwrap();
    
    // Verify it matches the original
    match deserialized {
        Command::PlaceBid { timestamp, bid: deserialized_bid } => {
            assert_eq!(timestamp, sample_bid_time());
            assert_eq!(deserialized_bid, bid);
        },
        _ => panic!("Expected PlaceBid command"),
    }
}

#[test]
fn test_command_success_serialization() {
    // AuctionAdded success
    let auction = sample_auction();
    let auction_added = CommandSuccess::AuctionAdded {
        timestamp: sample_starts_at(),
        auction: auction.clone(),
    };
    
    // Serialize to JSON
    let serialized = to_string(&auction_added).unwrap();
    
    // Verify it contains the expected data
    let json_value = serde_json::from_str::<serde_json::Value>(&serialized).unwrap();
    assert_eq!(json_value["$type"], "AuctionAdded");
    
    // Deserialize back
    let deserialized: CommandSuccess = from_str(&serialized).unwrap();
    
    // Verify it matches the original
    match deserialized {
        CommandSuccess::AuctionAdded { timestamp, auction: deserialized_auction } => {
            assert_eq!(timestamp, sample_starts_at());
            assert_eq!(deserialized_auction, auction);
        },
        _ => panic!("Expected AuctionAdded success"),
    }
    
    // BidAccepted success
    let bid = bid_1();
    let bid_accepted = CommandSuccess::BidAccepted {
        timestamp: sample_bid_time(),
        bid: bid.clone(),
    };
    
    // Serialize to JSON
    let serialized = to_string(&bid_accepted).unwrap();
    
    // Verify it contains the expected data
    let json_value = serde_json::from_str::<serde_json::Value>(&serialized).unwrap();
    assert_eq!(json_value["$type"], "BidAccepted");
    
    // Deserialize back
    let deserialized: CommandSuccess = from_str(&serialized).unwrap();
    
    // Verify it matches the original
    match deserialized {
        CommandSuccess::BidAccepted { timestamp, bid: deserialized_bid } => {
            assert_eq!(timestamp, sample_bid_time());
            assert_eq!(deserialized_bid, bid);
        },
        _ => panic!("Expected BidAccepted success"),
    }
}

#[test]
fn test_write_and_read_commands() {
    let test_file = "./test_commands.jsonl";
    
    // Create commands to write
    let auction = sample_auction();
    let add_auction = Command::AddAuction {
        timestamp: sample_starts_at(),
        auction: auction.clone(),
    };
    
    let bid = bid_1();
    let place_bid = Command::PlaceBid {
        timestamp: sample_bid_time(),
        bid: bid.clone(),
    };
    
    let commands = vec![add_auction, place_bid];
    
    // Write commands to file
    let write_result = write_commands(test_file, &commands);
    assert!(write_result.is_ok());
    
    // Read commands back from file
    let read_result = read_commands(test_file);
    assert!(read_result.is_ok());
    
    let read_commands = read_result.unwrap();
    assert_eq!(read_commands.len(), 2);
    
    // Clean up test file
    if Path::new(test_file).exists() {
        fs::remove_file(test_file).unwrap();
    }
}

#[test]
fn test_user_serialization() {
    // BuyerOrSeller
    let buyer = User::BuyerOrSeller {
        user_id: "user123".to_string(),
        name: "John Doe".to_string(),
    };
    
    let serialized = to_string(&buyer).unwrap();
    let json_value = serde_json::from_str::<serde_json::Value>(&serialized).unwrap();
    
    // Format should be a string with pipe separators
    assert_eq!(json_value, "BuyerOrSeller|user123|John Doe");
    
    let deserialized: User = from_str(&serialized).unwrap();
    match deserialized {
        User::BuyerOrSeller { user_id, name } => {
            assert_eq!(user_id, "user123");
            assert_eq!(name, "John Doe");
        },
        _ => panic!("Expected BuyerOrSeller"),
    }
    
    // Support
    let support = User::Support {
        user_id: "support123".to_string(),
    };
    
    let serialized = to_string(&support).unwrap();
    let json_value = serde_json::from_str::<serde_json::Value>(&serialized).unwrap();
    
    // Format should be a string with pipe separators
    assert_eq!(json_value, "Support|support123");
    
    let deserialized: User = from_str(&serialized).unwrap();
    match deserialized {
        User::Support { user_id } => {
            assert_eq!(user_id, "support123");
        },
        _ => panic!("Expected Support"),
    }
}