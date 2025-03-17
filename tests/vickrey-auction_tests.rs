use auction_site::domain::{
    AuctionType, Auction, Bid, User,
    single_sealed_bid::{Options as SBOptions, SingleSealedBidState as SBState},
    states::State,
    AuctionState, empty_state
};
use auction_site::money::{Amount, Currency};
use time::{macros::datetime, Duration, OffsetDateTime};
use std::str::FromStr;

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

fn buyer_2() -> User {
    User::BuyerOrSeller {
        user_id: "Buyer_2".to_string(),
        name: "Buyer 2".to_string(),
    }
}

fn sek(value: i64) -> Amount {
    Amount::new(Currency::SEK, value)
}

fn bid_amount_1() -> Amount {
    sek(10)
}

fn bid_1() -> Bid {
    Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_1(),
        at: sample_starts_at() + Duration::seconds(1),
        bid_amount: bid_amount_1(),
    }
}

fn bid_amount_2() -> Amount {
    sek(12)
}

fn bid_2() -> Bid {
    Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_2(),
        at: sample_starts_at() + Duration::seconds(2),
        bid_amount: bid_amount_2(),
    }
}

fn sample_vickrey_auction() -> Auction {
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

// Test that verifies state increment behavior
fn test_increment_spec(state: &SBState) {
    // Can increment twice
    let s = state.inc(sample_bid_time());
    let s2 = s.inc(sample_bid_time());
    assert_eq!(s, s2);

    // Won't end just after start
    let state = state.inc(sample_starts_at() + Duration::seconds(1));
    assert_eq!(state.has_ended(), false);

    // Won't end just before end
    let state = state.inc(sample_ends_at() - Duration::seconds(1));
    assert_eq!(state.has_ended(), false);

    // Won't end just before start
    let state = state.inc(sample_starts_at() - Duration::seconds(1));
    assert_eq!(state.has_ended(), false);

    // Will have ended just after end
    let state = state.inc(sample_ends_at() + Duration::seconds(1));
    assert_eq!(state.has_ended(), true);
}

#[test]
fn test_vickrey_auction_states() {
    let vickrey_auction = sample_vickrey_auction();
    let empty_vickrey_auction_state = match empty_state(&vickrey_auction) {
        AuctionState::SingleSealedBid(state) => state,
        _ => panic!("Expected SingleSealedBid state"),
    };
    
    // Can add bid to empty state
    let (state_with_1_bid, result_1) = empty_vickrey_auction_state.add_bid(bid_1());
    assert!(result_1.is_ok());
    
    // Can add second bid
    let (state_with_2_bids, result_2) = state_with_1_bid.add_bid(bid_2());
    assert!(result_2.is_ok());
    
    // Can end
    let state_ended_after_two_bids = state_with_2_bids.inc(sample_ends_at());
    
    // Verify the state is now DisclosingBids
    match &state_ended_after_two_bids {
        SBState::DisclosingBids { bids, expiry, options } => {
            // First bid should be highest (bid_2)
            assert_eq!(bids.len(), 2);
            assert_eq!(bids[0], bid_2());
            assert_eq!(bids[1], bid_1());
            assert_eq!(*expiry, sample_ends_at());
            assert_eq!(*options, SBOptions::Vickrey);
        },
        _ => panic!("Expected DisclosingBids state"),
    }
    
    // Can get winner and price from an ended auction
    let maybe_amount_and_winner = state_ended_after_two_bids.try_get_amount_and_winner();
    assert!(maybe_amount_and_winner.is_some());
    
    let (amount, winner) = maybe_amount_and_winner.unwrap();
    // In a Vickrey auction, winner pays second highest bid
    assert_eq!(amount, bid_amount_1());
    assert_eq!(winner, buyer_2().user_id().clone());
    
    // Test the increment spec
    test_increment_spec(&empty_vickrey_auction_state);
}

#[test]
fn test_vickrey_auction_with_only_one_bid() {
    let vickrey_auction = sample_vickrey_auction();
    let empty_vickrey_auction_state = match empty_state(&vickrey_auction) {
        AuctionState::SingleSealedBid(state) => state,
        _ => panic!("Expected SingleSealedBid state"),
    };
    
    // Add only one bid
    let (state_with_1_bid, result) = empty_vickrey_auction_state.add_bid(bid_1());
    assert!(result.is_ok());
    
    // End the auction
    let ended_state = state_with_1_bid.inc(sample_ends_at());
    
    // With only one bid, winner pays their own bid (no second price)
    let maybe_amount_and_winner = ended_state.try_get_amount_and_winner();
    assert!(maybe_amount_and_winner.is_some());
    
    let (amount, winner) = maybe_amount_and_winner.unwrap();
    assert_eq!(amount, bid_amount_1());
    assert_eq!(winner, buyer_1().user_id().clone());
}

#[test]
fn test_vickrey_auction_type_serialization() {
    // Sample type strings
    let vickrey_str = "Vickrey";
    let blind_str = "Blind";
    
    // Can deserialize Vickrey option
    let parsed_vickrey = SBOptions::from_str(vickrey_str).unwrap();
    assert_eq!(parsed_vickrey, SBOptions::Vickrey);
    
    // Can deserialize Blind option
    let parsed_blind = SBOptions::from_str(blind_str).unwrap();
    assert_eq!(parsed_blind, SBOptions::Blind);
    
    // Can serialize Vickrey option
    assert_eq!(SBOptions::Vickrey.to_string(), vickrey_str);
    
    // Can serialize Blind option
    assert_eq!(SBOptions::Blind.to_string(), blind_str);
}

#[test]
fn test_vickrey_auction_with_multiple_bids() {
    let vickrey_auction = sample_vickrey_auction();
    let empty_vickrey_auction_state = match empty_state(&vickrey_auction) {
        AuctionState::SingleSealedBid(state) => state,
        _ => panic!("Expected SingleSealedBid state"),
    };
    
    // Add three bids with different amounts
    let (state_with_1_bid, _) = empty_vickrey_auction_state.add_bid(bid_1()); // 10
    
    let bid_highest = Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_2(),
        at: sample_starts_at() + Duration::seconds(2),
        bid_amount: sek(20), // Highest bid
    };
    let (state_with_2_bids, _) = state_with_1_bid.add_bid(bid_highest);
    
    let bid_middle = Bid {
        for_auction: sample_auction_id(),
        bidder: User::BuyerOrSeller {
            user_id: "Buyer_3".to_string(),
            name: "Buyer 3".to_string(),
        },
        at: sample_starts_at() + Duration::seconds(3),
        bid_amount: sek(15), // Middle bid
    };
    let (state_with_3_bids, _) = state_with_2_bids.add_bid(bid_middle);
    
    // End the auction
    let ended_state = state_with_3_bids.inc(sample_ends_at());
    
    // Winner should be highest bidder, paying the second highest price
    let maybe_amount_and_winner = ended_state.try_get_amount_and_winner();
    assert!(maybe_amount_and_winner.is_some());
    
    let (amount, winner) = maybe_amount_and_winner.unwrap();
    assert_eq!(amount, sek(15)); // Second highest bid
    assert_eq!(winner, buyer_2().user_id().clone()); // Highest bidder
}