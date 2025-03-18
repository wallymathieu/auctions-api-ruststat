use auction_site::domain::{
    AuctionType, Auction, Bid, User,
    timed_ascending::{self},
    single_sealed_bid::Options as SBOptions,
    states::State
};
use auction_site::money::{Amount, Currency};
use time::{macros::datetime, Duration, OffsetDateTime};
// See https://users.rust-lang.org/t/sharing-code-and-macros-in-tests-directory/3098/7

// Sample data for tests
pub fn sample_auction_id() -> i64 {
    1
}

pub fn sample_title() -> String {
    "auction".to_string()
}

pub fn sample_starts_at() -> OffsetDateTime {
    datetime!(2016-01-01 8:28 UTC)
}

pub fn sample_ends_at() -> OffsetDateTime {
    datetime!(2016-02-01 8:28 UTC)
}

pub fn sample_bid_time() -> OffsetDateTime {
    datetime!(2016-01-15 8:28 UTC)
}

pub fn sample_seller() -> User {
    User::BuyerOrSeller {
        user_id: "Sample_Seller".to_string(),
        name: "Seller".to_string(),
    }
}

pub fn buyer_1() -> User {
    User::BuyerOrSeller {
        user_id: "Buyer_1".to_string(),
        name: "Buyer 1".to_string(),
    }
}

pub fn buyer_2() -> User {
    User::BuyerOrSeller {
        user_id: "Buyer_2".to_string(),
        name: "Buyer 2".to_string(),
    }
}

pub fn buyer_3() -> User {
    User::BuyerOrSeller {
        user_id: "Buyer_3".to_string(),
        name: "Buyer 3".to_string(),
    }
}

pub fn sek(value: i64) -> Amount {
    Amount::new(Currency::SEK, value)
}

pub fn vac(value: i64) -> Amount {
    Amount::new(Currency::VAC, value)
}


pub fn bid_amount_1() -> Amount {
    sek(10)
}

pub fn bid_1() -> Bid {
    Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_1(),
        at: sample_starts_at() + Duration::seconds(1),
        bid_amount: bid_amount_1(),
    }
}

pub fn bid_amount_2() -> Amount {
    sek(12)
}

pub fn bid_2() -> Bid {
    Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_2(),
        at: sample_starts_at() + Duration::seconds(2),
        bid_amount: bid_amount_2(),
    }
}

pub fn bid_less_than_2() -> Bid {
    Bid {
        for_auction: sample_auction_id(),
        bidder: buyer_3(),
        at: sample_starts_at() + Duration::seconds(3),
        bid_amount: sek(11), // Less than bid_2
    }
}

pub fn sample_auction_of_type(typ: AuctionType) -> Auction {
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

pub fn sample_timed_asc_auction() -> Auction {
    sample_auction_of_type(AuctionType::TimedAscending(timed_ascending::Options::default_options(Currency::SEK)))
}

pub fn sample_vickrey_auction() -> Auction {
    sample_auction_of_type(AuctionType::SingleSealedBid(SBOptions::Vickrey))
}
pub fn sample_blind_auction() -> Auction {
    sample_auction_of_type(AuctionType::SingleSealedBid(SBOptions::Blind))
}

pub fn test_increment_state<S: State + Clone + PartialEq+ std::fmt::Debug>(base_state: &S) {
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

// Test that verifies state increment behavior
pub fn test_increment_spec<T: State + Clone+ PartialEq + std::fmt::Debug>(state: &T) {
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
