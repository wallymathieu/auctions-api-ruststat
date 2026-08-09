use actix_web::http::StatusCode;
use actix_web::{test, web, App};
use auction_site::domain::single_sealed_bid::Options as SBOptions;
use auction_site::domain::states::State;
use auction_site::domain::{empty_state, timed_ascending, Auction, AuctionType, Bid, User};
use auction_site::money::{AmountValue, Currency};
use auction_site::web::app::{configure_app, init_app_state};
use auction_site::web::types::AppState;
use serde_json::Value;
use time::{Duration, OffsetDateTime};
#[path = "utils/mod.rs"]
mod utils;
use utils::*;

fn auction_of_type(typ: AuctionType, starts_at: OffsetDateTime, expiry: OffsetDateTime) -> Auction {
    Auction {
        auction_id: sample_auction_id(),
        title: sample_title(),
        starts_at,
        expiry,
        seller: sample_seller(),
        auction_currency: Currency::SEK,
        typ,
    }
}

// An auction that started two hours ago and expired one hour ago
fn expired_auction_of_type(typ: AuctionType) -> Auction {
    let now = OffsetDateTime::now_utc();
    auction_of_type(typ, now - Duration::hours(2), now - Duration::hours(1))
}

// An auction that started an hour ago and expires in an hour
fn running_auction_of_type(typ: AuctionType) -> Auction {
    let now = OffsetDateTime::now_utc();
    auction_of_type(typ, now - Duration::hours(1), now + Duration::hours(1))
}

fn timed_ascending_auction_with_reserve(reserve_price: AmountValue) -> Auction {
    expired_auction_of_type(AuctionType::TimedAscending(timed_ascending::Options {
        reserve_price,
        ..timed_ascending::Options::default_options()
    }))
}

// A bid placed while the auction was still running
fn bid_before_expiry(auction: &Auction, bidder: User, amount: AmountValue, before: Duration) -> Bid {
    Bid {
        for_auction: auction.auction_id,
        bidder,
        at: auction.starts_at + Duration::minutes(1) + before,
        bid_amount: amount,
    }
}

// Seed the repository with a state that has not been advanced past the expiry,
// which is what the repository looks like when no bid arrives after the expiry.
fn seeded_state(auction: Auction, bids: Vec<Bid>) -> AppState {
    let mut state = empty_state(&auction);
    for bid in bids {
        let (next, result) = state.add_bid(bid);
        assert!(result.is_ok(), "seed bid was rejected: {:?}", result.err());
        state = next;
    }
    assert!(
        !state.has_ended(),
        "seeded state should still be running, so that the read path has to advance it"
    );

    let app_state = init_app_state();
    app_state
        .lock()
        .unwrap()
        .insert(auction.auction_id, (auction, state));
    app_state
}

async fn get_auction(app_state: &AppState, auction_id: i64) -> (StatusCode, Value) {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_app),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/auctions/{}", auction_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    (status, test::read_body_json(resp).await)
}

async fn get_auction_ok(app_state: &AppState, auction_id: i64) -> Value {
    let (status, json) = get_auction(app_state, auction_id).await;
    assert_eq!(status, StatusCode::OK, "unexpected status, body: {}", json);
    json
}

fn stored_state_has_ended(app_state: &AppState, auction_id: i64) -> bool {
    app_state
        .lock()
        .unwrap()
        .get(&auction_id)
        .map(|(_, state)| state.has_ended())
        .expect("auction should still be in the repository")
}

fn amounts(json: &Value) -> Vec<Value> {
    json["bids"]
        .as_array()
        .unwrap_or_else(|| panic!("expected a bids array, got {}", json))
        .iter()
        .map(|bid| bid["amount"].clone())
        .collect()
}

#[actix_web::test]
async fn test_get_auction_reports_winner_for_expired_timed_ascending_auction() {
    let auction = expired_auction_of_type(AuctionType::TimedAscending(
        timed_ascending::Options::default_options(),
    ));
    let auction_id = auction.auction_id;
    let bid = bid_before_expiry(&auction, buyer_1(), bid_amount_1(), Duration::ZERO);
    let app_state = seeded_state(auction, vec![bid]);

    let json = get_auction_ok(&app_state, auction_id).await;

    assert_eq!(
        amounts(&json),
        vec![Value::from(bid_amount_1())],
        "expected the bid to be listed, got {}",
        json
    );
    assert_eq!(
        json["bids"][0]["bidder"],
        Value::from(buyer_1().to_string()),
        "expected the bidder to be disclosed, got {}",
        json
    );
    assert_eq!(
        json["winner"],
        Value::from(buyer_1().user_id().as_str()),
        "expected a winner, got {}",
        json
    );
    assert_eq!(
        json["winnerPrice"],
        Value::from(sek(bid_amount_1()).to_string()),
        "expected a winner price, got {}",
        json
    );
}

#[actix_web::test]
async fn test_get_auction_discloses_bids_for_expired_single_sealed_bid_auction() {
    let auction = expired_auction_of_type(AuctionType::SingleSealedBid(SBOptions::Blind));
    let auction_id = auction.auction_id;
    let bid = bid_before_expiry(&auction, buyer_1(), bid_amount_1(), Duration::ZERO);
    let app_state = seeded_state(auction, vec![bid]);

    let json = get_auction_ok(&app_state, auction_id).await;

    assert_eq!(
        amounts(&json),
        vec![Value::from(bid_amount_1())],
        "expected disclosed bids, got {}",
        json
    );
    assert_eq!(
        json["winner"],
        Value::from(buyer_1().user_id().as_str()),
        "expected a winner, got {}",
        json
    );
    assert_eq!(
        json["winnerPrice"],
        Value::from(sek(bid_amount_1()).to_string()),
        "expected the blind winner to pay their own bid, got {}",
        json
    );
}

// The read path must write the advanced state back, not just advance a local copy.
#[actix_web::test]
async fn test_get_auction_persists_the_advanced_state() {
    let auction = expired_auction_of_type(AuctionType::SingleSealedBid(SBOptions::Blind));
    let auction_id = auction.auction_id;
    let bid = bid_before_expiry(&auction, buyer_1(), bid_amount_1(), Duration::ZERO);
    let app_state = seeded_state(auction, vec![bid]);

    assert!(!stored_state_has_ended(&app_state, auction_id));

    let _ = get_auction_ok(&app_state, auction_id).await;

    assert!(
        stored_state_has_ended(&app_state, auction_id),
        "reading an expired auction should have advanced the stored state"
    );
}

#[actix_web::test]
async fn test_get_auction_pays_second_price_for_expired_vickrey_auction() {
    let auction = expired_auction_of_type(AuctionType::SingleSealedBid(SBOptions::Vickrey));
    let auction_id = auction.auction_id;
    let low = bid_before_expiry(&auction, buyer_1(), bid_amount_1(), Duration::ZERO);
    let high = bid_before_expiry(&auction, buyer_2(), bid_amount_2(), Duration::seconds(1));
    let app_state = seeded_state(auction, vec![low, high]);

    let json = get_auction_ok(&app_state, auction_id).await;

    assert_eq!(
        amounts(&json),
        vec![Value::from(bid_amount_2()), Value::from(bid_amount_1())],
        "expected bids disclosed highest first, got {}",
        json
    );
    assert_eq!(
        json["winner"],
        Value::from(buyer_2().user_id().as_str()),
        "expected the highest bidder to win, got {}",
        json
    );
    assert_eq!(
        json["winnerPrice"],
        Value::from(sek(bid_amount_1()).to_string()),
        "expected the winner to pay the second highest bid, got {}",
        json
    );
}

// Sealed bids must stay sealed while the auction is still accepting them.
#[actix_web::test]
async fn test_get_auction_hides_bids_for_running_single_sealed_bid_auction() {
    let auction = running_auction_of_type(AuctionType::SingleSealedBid(SBOptions::Blind));
    let auction_id = auction.auction_id;
    let bid = bid_before_expiry(&auction, buyer_1(), bid_amount_1(), Duration::ZERO);
    let app_state = seeded_state(auction, vec![bid]);

    let json = get_auction_ok(&app_state, auction_id).await;

    assert_eq!(amounts(&json), Vec::<Value>::new(), "bids leaked: {}", json);
    assert_eq!(json["winner"], Value::Null, "winner leaked: {}", json);
    assert_eq!(json["winnerPrice"], Value::Null, "price leaked: {}", json);
    assert!(
        !stored_state_has_ended(&app_state, auction_id),
        "a running auction must not be advanced past its expiry"
    );
}

#[actix_web::test]
async fn test_get_auction_reports_no_winner_for_running_timed_ascending_auction() {
    let auction = running_auction_of_type(AuctionType::TimedAscending(
        timed_ascending::Options::default_options(),
    ));
    let auction_id = auction.auction_id;
    let bid = bid_before_expiry(&auction, buyer_1(), bid_amount_1(), Duration::ZERO);
    let app_state = seeded_state(auction, vec![bid]);

    let json = get_auction_ok(&app_state, auction_id).await;

    assert_eq!(
        amounts(&json),
        vec![Value::from(bid_amount_1())],
        "an English auction shows its bids while running, got {}",
        json
    );
    assert_eq!(json["winner"], Value::Null, "winner leaked: {}", json);
    assert_eq!(json["winnerPrice"], Value::Null, "price leaked: {}", json);
}

#[actix_web::test]
async fn test_get_auction_reports_no_winner_when_reserve_price_is_not_met() {
    let auction = timed_ascending_auction_with_reserve(bid_amount_1() + 1);
    let auction_id = auction.auction_id;
    let bid = bid_before_expiry(&auction, buyer_1(), bid_amount_1(), Duration::ZERO);
    let app_state = seeded_state(auction, vec![bid]);

    let json = get_auction_ok(&app_state, auction_id).await;

    assert_eq!(
        amounts(&json),
        vec![Value::from(bid_amount_1())],
        "the bid is still listed, got {}",
        json
    );
    assert_eq!(json["winner"], Value::Null, "expected no winner, got {}", json);
    assert_eq!(json["winnerPrice"], Value::Null, "expected no price, got {}", json);
}

#[actix_web::test]
async fn test_get_auction_returns_not_found_for_unknown_auction() {
    let app_state = init_app_state();

    let (status, json) = get_auction(&app_state, sample_auction_id()).await;

    assert_eq!(status, StatusCode::NOT_FOUND, "got {}", json);
    assert_eq!(json["message"], Value::from("Auction not found"));
}
