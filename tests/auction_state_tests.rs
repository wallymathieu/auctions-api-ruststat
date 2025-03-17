use auction_site::domain::{
    AuctionType, Auction, Bid, User,
    single_sealed_bid::{Options as SBOptions, SingleSealedBidState as SBState},
    timed_ascending::{Options as TAOptions, TimedAscendingState as TAState},
    states::State,
    AuctionState
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
    sample_starts_at() + Duration::seconds(10)
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

fn buyer_3() -> User {
    User::BuyerOrSeller {
        user_id: "Buyer_3".to_string(),
        name: "Buyer 3".to_string(),
    }
}

fn sek(value: i64) -> Amount {
    Amount::new(Currency::SEK, value)
}

fn bid_1() -> Bid {
    Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_1(),
        at: sample_starts_at() + Duration::seconds(1),
        bid_amount: sek(10),
    }
}

fn bid_2() -> Bid {
    Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_2(),
        at: sample_starts_at() + Duration::seconds(2),
        bid_amount: sek(12),
    }
}

fn bid_less_than_2() -> Bid {
    Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_3(),
        at: sample_starts_at() + Duration::seconds(3),
        bid_amount: sek(11), // Less than bid_2
    }
}

fn sample_auction_of_type(typ: AuctionType) -> Auction {
    Auction {
        auction_id: sample_auction_id(),
        title: sample_title(),
        starts_at: sample_starts_at(),
        expiry: sample_ends_at(),
        seller: sample_seller(),
        auction_currency: Currency::SEK,
        typ,
    }
}

fn test_increment_state<S: State + Clone + PartialEq+ std::fmt::Debug>(base_state: &S) {
    // Can increment twice
    let s = base_state.inc(sample_bid_time());
    let s2 = s.inc(sample_bid_time());
    assert_eq!(s, s2);

    // Won't end just after start
    let state = base_state.inc(sample_starts_at() + Duration::seconds(1));
    assert_eq!(state.has_ended(), false);

    // Won't end just before end
    let state = base_state.inc(sample_ends_at() - Duration::seconds(1));
    assert_eq!(state.has_ended(), false);

    // Won't end just before start
    let state = base_state.inc(sample_starts_at() - Duration::seconds(1));
    assert_eq!(state.has_ended(), false);

    // Will have ended just after end
    let state = base_state.inc(sample_ends_at() + Duration::seconds(1));
    assert_eq!(state.has_ended(), true);
}

// Single Sealed Bid (Blind) auction tests
#[test]
fn test_blind_auction_states() {
    let blind_auction = sample_auction_of_type(AuctionType::SingleSealedBid(SBOptions::Blind));
    let empty_state = match auction_site::domain::empty_state(&blind_auction) {
        AuctionState::SingleSealedBid(state) => state,
        _ => panic!("Expected SingleSealedBid state"),
    };
    
    // Can add bid to empty state
    let (state_with_1_bid, result_1) = empty_state.add_bid(bid_1());
    assert!(result_1.is_ok());
    
    // Can add second bid
    let (state_with_2_bids, result_2) = state_with_1_bid.add_bid(bid_2());
    assert!(result_2.is_ok());
    
    // Can end
    let state_ended_after_two_bids = state_with_2_bids.inc(sample_ends_at());
    match state_ended_after_two_bids {
        SBState::DisclosingBids { ref bids, .. } => {
            assert_eq!(bids.len(), 2);
            assert_eq!(bids[0].bid_amount.clone(), sek(12)); // Higher bid first
            assert_eq!(bids[1].bid_amount.clone(), sek(10));
        },
        _ => panic!("Expected DisclosingBids state"),
    }
    
    // Can get winner and price from an ended auction
    let winner_and_price = state_ended_after_two_bids.try_get_amount_and_winner();
    assert!(winner_and_price.is_some());
    let (amount, winner) = winner_and_price.unwrap();
    assert_eq!(amount, sek(12));
    assert_eq!(winner, "Buyer_2");
    
    // Test base increment state functionality
    test_increment_state(&empty_state);
}

// Single Sealed Bid (Vickrey) auction tests
#[test]
fn test_vickrey_auction_states() {
    let vickrey_auction = sample_auction_of_type(AuctionType::SingleSealedBid(SBOptions::Vickrey));
    let empty_state = match auction_site::domain::empty_state(&vickrey_auction) {
        AuctionState::SingleSealedBid(state) => state,
        _ => panic!("Expected SingleSealedBid state"),
    };
    
    // Can add bid to empty state
    let (state_with_1_bid, result_1) = empty_state.add_bid(bid_1());
    assert!(result_1.is_ok());
    
    // Can add second bid
    let (state_with_2_bids, result_2) = state_with_1_bid.add_bid(bid_2());
    assert!(result_2.is_ok());
    
    // Can end
    let state_ended_after_two_bids = state_with_2_bids.inc(sample_ends_at());
    
    // Can get winner and price from an ended auction (in Vickrey, winner pays second highest price)
    let winner_and_price = state_ended_after_two_bids.try_get_amount_and_winner();
    assert!(winner_and_price.is_some());
    let (amount, winner) = winner_and_price.unwrap();
    assert_eq!(amount, sek(10)); // Second highest bid
    assert_eq!(winner, "Buyer_2"); // Highest bidder
    
    // Test base increment state functionality
    test_increment_state(&empty_state);
}

// Timed Ascending (English) auction tests
#[test]
fn test_english_auction_states() {
    let timed_asc_auction = sample_auction_of_type(AuctionType::TimedAscending(
        TAOptions::default_options(Currency::SEK)
    ));
    
    let empty_state = match auction_site::domain::empty_state(&timed_asc_auction) {
        AuctionState::TimedAscending(state) => state,
        _ => panic!("Expected TimedAscending state"),
    };
    
    // Can add bid to empty state (after transitioning to OnGoing)
    let ongoing_state = empty_state.inc(sample_starts_at() + Duration::seconds(1));
    let (state_with_1_bid, result_1) = ongoing_state.add_bid(bid_1());
    assert!(result_1.is_ok());
    
    // Can add second bid
    let (state_with_2_bids, result_2) = state_with_1_bid.add_bid(bid_2());
    assert!(result_2.is_ok());
    
    // Can end
    let empty_ended_state = empty_state.inc(sample_ends_at() + Duration::seconds(1));
    match empty_ended_state {
        TAState::HasEnded { bids, .. } => {
            assert_eq!(bids.len(), 0);
        },
        _ => panic!("Expected HasEnded state"),
    }
    
    // Ended with two bids
    let state_ended_after_two_bids = state_with_2_bids.inc(sample_ends_at());
    match state_ended_after_two_bids {
        TAState::HasEnded { ref bids, .. } => {
            assert_eq!(bids.len(), 2);
            assert_eq!(bids[0].bid_amount.clone(), sek(12)); // Higher bid first
            assert_eq!(bids[1].bid_amount.clone(), sek(10));
        },
        _ => panic!("Expected HasEnded state"),
    }
    
    // Can get winner and price from an auction
    let winner_and_price = state_ended_after_two_bids.try_get_amount_and_winner();
    assert!(winner_and_price.is_some());
    let (amount, winner) = winner_and_price.unwrap();
    assert_eq!(amount, sek(12));
    assert_eq!(winner, "Buyer_2");
    
    // Can't place bid lower than highest bid
    let (_, maybe_fail) = state_with_2_bids.add_bid(bid_less_than_2());
    assert!(maybe_fail.is_err());
    
    // Test base increment state functionality
    test_increment_state(&empty_state);
}

#[test]
fn test_english_auction_type_serialization() {
    // Sample type string
    let sample_type_str = "English|VAC0|VAC0|0";
    let sample_type = TAOptions::default_options(Currency::VAC);
    
    // Can deserialize sample type
    let parsed = TAOptions::from_str(sample_type_str).unwrap();
    assert_eq!(parsed.reserve_price, sample_type.reserve_price);
    assert_eq!(parsed.min_raise, sample_type.min_raise);
    assert_eq!(parsed.time_frame, sample_type.time_frame);
    
    // Can serialize sample type
    assert_eq!(sample_type.to_string(), sample_type_str);
    
    // Sample with values
    let sample_with_values_str = "English|VAC10|VAC20|30";
    let sample_with_values = TAOptions {
        reserve_price: Amount::new(Currency::VAC, 10),
        min_raise: Amount::new(Currency::VAC, 20),
        time_frame: Duration::seconds(30),
    };
    
    // Can deserialize sample with values
    let parsed = TAOptions::from_str(sample_with_values_str).unwrap();
    assert_eq!(parsed.reserve_price, sample_with_values.reserve_price);
    assert_eq!(parsed.min_raise, sample_with_values.min_raise);
    assert_eq!(parsed.time_frame, sample_with_values.time_frame);
    
    // Can serialize sample with values
    assert_eq!(sample_with_values.to_string(), sample_with_values_str);
}