use auction_site::domain::{
    Bid, single_sealed_bid::{Options as SBOptions, SingleSealedBidState as SBState},
    states::State,
    AuctionState, empty_state
};
use time::Duration;
#[path="utils/mod.rs"] mod utils;
use utils::*;
// Sample data for tests

#[test]
fn test_blind_auction_states() {
    let blind_auction = sample_blind_auction();
    let empty_blind_auction_state = match empty_state(&blind_auction) {
        AuctionState::SingleSealedBid(state) => state,
        _ => panic!("Expected SingleSealedBid state"),
    };

    // Can add bid to empty state
    let (state_with_1_bid, result_1) = empty_blind_auction_state.add_bid(bid_1());
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
            assert_eq!(*options, SBOptions::Blind);
        },
        _ => panic!("Expected DisclosingBids state"),
    }

    // Can get winner and price from an ended auction
    let maybe_amount_and_winner = state_ended_after_two_bids.try_get_amount_and_winner();
    assert!(maybe_amount_and_winner.is_some());

    let (amount, winner) = maybe_amount_and_winner.unwrap();
    // In a blind auction, winner pays their own bid (highest)
    assert_eq!(amount, bid_amount_2());
    assert_eq!(winner, buyer_2().user_id().clone());

    // Test the increment spec
    test_increment_spec(&empty_blind_auction_state);
}

#[test]
fn test_cannot_place_duplicate_bids() {
    let blind_auction = sample_blind_auction();
    let empty_blind_auction_state = match empty_state(&blind_auction) {
        AuctionState::SingleSealedBid(state) => state,
        _ => panic!("Expected SingleSealedBid state"),
    };

    // First bid is accepted
    let (state_with_bid, result) = empty_blind_auction_state.add_bid(bid_1());
    assert!(result.is_ok());

    // Same bidder cannot place a second bid
    let duplicate_bid = Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_1(), // Same bidder
        at: sample_bid_time(),
        bid_amount: 15, // Different amount
    };

    let (_, result) = state_with_bid.add_bid(duplicate_bid);
    assert!(result.is_err());

    // Verify the error is AlreadyPlacedBid
    match result {
        Err(auction_site::domain::core::Errors::AlreadyPlacedBid) => {},
        _ => panic!("Expected AlreadyPlacedBid error"),
    }
}

#[test]
fn test_cannot_bid_after_end() {
    let blind_auction = sample_blind_auction();
    let empty_blind_auction_state = match empty_state(&blind_auction) {
        AuctionState::SingleSealedBid(state) => state,
        _ => panic!("Expected SingleSealedBid state"),
    };

    // Advance state to ended
    let ended_state = empty_blind_auction_state.inc(sample_ends_at() + Duration::seconds(1));

    // Try to place a bid after end
    let late_bid = Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_1(),
        at: sample_ends_at() + Duration::seconds(2),
        bid_amount: 10,
    };

    let (_, result) = ended_state.add_bid(late_bid);

    // Should get AuctionHasEnded error
    assert!(result.is_err());
    match result {
        Err(auction_site::domain::core::Errors::AuctionHasEnded(id)) => {
            assert_eq!(id, sample_auction_id());
        },
        _ => panic!("Expected AuctionHasEnded error"),
    }
}
