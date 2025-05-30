use auction_site::domain::{
    AuctionType, Auction, Bid,
    timed_ascending::{self, TimedAscendingState as TAState},
    states::State,
    AuctionState, empty_state,
    core::Errors
};
use auction_site::money::{Amount, Currency};
use time::Duration;
use std::str::FromStr;
#[path="utils/mod.rs"] mod utils;
use utils::*;

#[test]
fn test_english_auction_states() {
    let timed_asc_auction = sample_timed_asc_auction();
    let empty_asc_auction_state = match empty_state(&timed_asc_auction) {
        AuctionState::TimedAscending(state) => state,
        _ => panic!("Expected TimedAscending state"),
    };

    // Start the auction
    let started_state = empty_asc_auction_state.inc(sample_starts_at() + Duration::seconds(1));

    // Can add bid to empty state
    let (state_with_1_bid, result_1) = started_state.add_bid(bid_1());
    assert!(result_1.is_ok());

    // Can add second bid
    let (state_with_2_bids, result_2) = state_with_1_bid.add_bid(bid_2());
    assert!(result_2.is_ok());

    // Can end empty auction
    let empty_ended_asc_auction_state = empty_asc_auction_state.inc(sample_ends_at() + Duration::seconds(1));
    match empty_ended_asc_auction_state {
        TAState::HasEnded { bids, .. } => {
            assert_eq!(bids.len(), 0);
        },
        _ => panic!("Expected HasEnded state"),
    }

    // Can end auction with bids
    let state_ended_after_two_bids = state_with_2_bids.inc(sample_ends_at());
    match state_ended_after_two_bids {
        TAState::HasEnded { ref bids, .. } => {
            assert_eq!(bids.len(), 2);
            assert_eq!(bids[0], bid_2());
            assert_eq!(bids[1], bid_1());
        },
        _ => panic!("Expected HasEnded state"),
    }

    // Can't bid after auction has ended
    let (_, err_after_ended) = state_ended_after_two_bids.add_bid(bid_less_than_2());
    assert!(err_after_ended.is_err());
    match err_after_ended {
        Err(Errors::AuctionHasEnded(id)) => {
            assert_eq!(id, sample_auction_id());
        },
        _ => panic!("Expected AuctionHasEnded error"),
    }

    // Can get winner and price from an auction
    let maybe_amount_and_winner = state_ended_after_two_bids.try_get_amount_and_winner();
    assert!(maybe_amount_and_winner.is_some());
    let (amount, winner) = maybe_amount_and_winner.unwrap();
    assert_eq!(amount, bid_amount_2());
    assert_eq!(winner, buyer_2().user_id().clone());

    // Can't place bid lower than highest bid + minimum raise
    let (_, maybe_fail) = state_with_2_bids.add_bid(bid_less_than_2());
    assert!(maybe_fail.is_err());
    match maybe_fail {
        Err(Errors::MustPlaceBidOverHighestBid(amount)) => {
            assert_eq!(amount, bid_amount_2());
        },
        _ => panic!("Expected MustPlaceBidOverHighestBid error"),
    }

    // Test the increment spec
    test_increment_spec(&empty_asc_auction_state);
}

#[test]
fn test_english_auction_type_serialization() {
    // Sample type string
    let sample_type_str = "English|0|0|0";
    let sample_type = timed_ascending::Options::default_options();

    // Can deserialize sample type
    let parsed = timed_ascending::Options::from_str(sample_type_str).unwrap();
    assert_eq!(parsed.reserve_price, sample_type.reserve_price);
    assert_eq!(parsed.min_raise, sample_type.min_raise);
    assert_eq!(parsed.time_frame, sample_type.time_frame);

    // Can serialize sample type
    assert_eq!(sample_type.to_string(), sample_type_str);

    // Sample with values
    let sample_with_values_type_str = "English|10|20|30";
    let sample_with_values_type = timed_ascending::Options {
        reserve_price: 10,
        min_raise: 20,
        time_frame: Duration::seconds(30),
    };

    // Can deserialize sample with values type
    let parsed = timed_ascending::Options::from_str(sample_with_values_type_str).unwrap();
    assert_eq!(parsed.reserve_price, sample_with_values_type.reserve_price);
    assert_eq!(parsed.min_raise, sample_with_values_type.min_raise);
    assert_eq!(parsed.time_frame, sample_with_values_type.time_frame);

    // Can serialize sample with values type
    assert_eq!(sample_with_values_type.to_string(), sample_with_values_type_str);
}

#[test]
fn test_english_auction_with_reserve_price() {
    // Create auction with reserve price
    let auction_with_reserve = Auction {
        auction_id: sample_auction_id(),
        title: sample_title(),
        starts_at: sample_starts_at(),
        expiry: sample_ends_at(),
        seller: sample_seller(),
        auction_currency: Currency::SEK,
        typ: AuctionType::TimedAscending(
            timed_ascending::Options {
                reserve_price: 15, // Reserve price higher than bids
                min_raise: 0,
                time_frame: Duration::seconds(0),
            }
        ),
    };

    let state = match empty_state(&auction_with_reserve) {
        AuctionState::TimedAscending(state) => state,
        _ => panic!("Expected TimedAscending state"),
    };

    // Start auction
    let started_state = state.inc(sample_starts_at() + Duration::seconds(1));

    // Add some bids
    let (state_with_bid, _) = started_state.add_bid(bid_2()); // bid of 12

    // End auction
    let ended_state = state_with_bid.inc(sample_ends_at() + Duration::seconds(1));

    // No winner because bid was below reserve price
    let maybe_winner = ended_state.try_get_amount_and_winner();
    assert!(maybe_winner.is_none());
}

#[test]
fn test_english_auction_with_min_raise() {
    // Create auction with minimum raise requirement
    let auction_with_min_raise = Auction {
        auction_id: sample_auction_id(),
        title: sample_title(),
        starts_at: sample_starts_at(),
        expiry: sample_ends_at(),
        seller: sample_seller(),
        auction_currency: Currency::SEK,
        typ: AuctionType::TimedAscending(
            timed_ascending::Options {
                reserve_price: 0,
                min_raise: 5, // Require bids to be at least 5 higher than current
                time_frame: Duration::seconds(0),
            }
        ),
    };

    let state = match empty_state(&auction_with_min_raise) {
        AuctionState::TimedAscending(state) => state,
        _ => panic!("Expected TimedAscending state"),
    };

    // Start auction
    let started_state = state.inc(sample_starts_at() + Duration::seconds(1));

    // First bid is fine
    let (state_with_bid, result) = started_state.add_bid(bid_1()); // bid of 10
    assert!(result.is_ok(), "{:?}", result);

    // Second bid must be at least 5 more than first (10 + 5 = 15)
    let small_raise_bid = Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_2(),
        at: sample_starts_at() + Duration::seconds(2),
        bid_amount: 14, // Only 4 more than first bid
    };

    let (_, result) = state_with_bid.add_bid(small_raise_bid);
    assert!(result.is_err(), "{:?}", result);

    // Bid with sufficient raise
    let sufficient_raise_bid = Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_2(),
        at: sample_starts_at() + Duration::seconds(2),
        bid_amount: 15, // 5 more than first bid
    };

    let (state_with_second_bid, result_s) = state_with_bid.add_bid(sufficient_raise_bid);
    assert!(result_s.is_ok(), "{:?}", result_s);

    // Verify the bid was accepted
    let bids = state_with_second_bid.get_bids();
    assert_eq!(bids.len(), 2);
    assert_eq!(bids[0].bid_amount, 15);
}

#[test]
fn test_auction_extends_when_bids_placed_near_end() {
    // Create auction with time extension
    let auction_with_extension = Auction {
        auction_id: sample_auction_id(),
        title: sample_title(),
        starts_at: sample_starts_at(),
        expiry: sample_ends_at(),
        seller: sample_seller(),
        auction_currency: Currency::SEK,
        typ: AuctionType::TimedAscending(
            timed_ascending::Options {
                reserve_price: 0,
                min_raise: 0,
                time_frame: Duration::minutes(5), // 5 minute extension when bid placed
            }
        ),
    };

    let state = match empty_state(&auction_with_extension) {
        AuctionState::TimedAscending(state) => state,
        _ => panic!("Expected TimedAscending state"),
    };

    // Start auction
    let started_state = state.inc(sample_starts_at() + Duration::seconds(1));

    // Place bid near the end
    let almost_ending_time = sample_ends_at() - Duration::seconds(30);
    let near_end_bid = Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_1(),
        at: almost_ending_time,
        bid_amount: 10,
    };

    let (state_with_bid, result) = started_state.add_bid(near_end_bid);
    assert!(result.is_ok());

    // Check that the auction hasn't ended at the original end time
    let state_at_original_end = state_with_bid.inc(sample_ends_at());
    assert!(!state_at_original_end.has_ended());

    // But it should end 5 minutes after the bid time
    let extended_end_time = almost_ending_time + Duration::minutes(5) + Duration::seconds(1);
    let state_after_extension = state_at_original_end.inc(extended_end_time);
    assert!(state_after_extension.has_ended());
}
