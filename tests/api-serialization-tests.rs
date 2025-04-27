use auction_site::domain::{AuctionType, Auction};
use auction_site::domain::timed_ascending::Options as TAOptions;
use auction_site::money::Currency;
use auction_site::web::types::{AddAuctionRequest, BidRequest};
use serde_json::json;
#[path="utils/mod.rs"] mod utils;
use utils::*;

#[test]
fn test_auction_request_deserialization() {
    // Create a JSON representation of an auction request
    let json_data = json!({
        "id": 1,
        "startsAt": "2016-01-01T08:28:00.000Z",
        "endsAt": "2016-02-01T08:28:00.000Z",
        "title": "First auction"
    });
    
    // Deserialize to AddAuctionRequest
    let request: AddAuctionRequest = serde_json::from_value(json_data).unwrap();
    
    // Verify fields
    assert_eq!(request.id, sample_auction_id());
    assert_eq!(request.starts_at, sample_starts_at());
    assert_eq!(request.ends_at, sample_ends_at());
    assert_eq!(request.title, "First auction");
    
    // Create an auction from the request
    let auction = request.to_auction(sample_seller());
    
    // Verify the auction has correct default values for non-specified fields
    assert_eq!(auction.auction_currency, Currency::VAC);
    
    // Verify auction type is TimedAscending with default options
    match auction.typ {
        AuctionType::TimedAscending(options) => {
            assert_eq!(options.reserve_price, 0);
            assert_eq!(options.min_raise, 0);
        },
        _ => panic!("Expected TimedAscending auction type"),
    }
}

#[test]
fn test_auction_request_with_currency_deserialization() {
    // Create a JSON representation of an auction request with currency
    let json_data = json!({
        "id": 1,
        "startsAt": "2016-01-01T00:00:00.000Z",
        "endsAt": "2016-02-01T00:00:00.000Z",
        "title": "First auction",
        "currency": "SEK"
    });
    
    // Deserialize to AddAuctionRequest
    let request: AddAuctionRequest = serde_json::from_value(json_data).unwrap();
    
    // Create an auction from the request
    let auction = request.to_auction(sample_seller());
    
    // Verify the auction has the specified currency
    assert_eq!(auction.auction_currency, Currency::SEK);
}

#[test]
fn test_bid_request_deserialization() {
    // Create a JSON representation of a bid request
    let json_data = json!({
        "amount": 10
    });
    
    // Deserialize to BidRequest
    let request: BidRequest = serde_json::from_value(json_data).unwrap();
    
    // Verify fields
    assert_eq!(request.amount, 10);
}

#[test]
fn test_auction_serialization() {
    // Create an auction
    let auction = Auction {
        auction_id: sample_auction_id(),
        starts_at: sample_starts_at(),
        title: "First auction".to_string(),
        expiry: sample_ends_at(),
        seller: sample_seller(),
        auction_currency: Currency::VAC,
        typ: AuctionType::TimedAscending(TAOptions::default_options()),
    };
    
    // Serialize to JSON
    let json = serde_json::to_value(&auction).unwrap();
    
    // Verify serialized format
    assert_eq!(json["id"], json!(1), "id {:?}", json["id"]);
    assert_eq!(json["startsAt"], json!("2016-01-01T08:28:00Z"), "startsAt {:?}", json["startsAt"]);
    assert_eq!(json["title"], json!("First auction"), "title {:?}", json["title"]);
    assert_eq!(json["expiry"], json!("2016-02-01T08:28:00Z"), "expiry {:?}", json["expiry"]);
    assert_eq!(json["user"], json!("BuyerOrSeller|Sample_Seller|Seller"), "user {:?}", json["user"]);
    assert_eq!(json["currency"], json!("VAC"), "currency {:?}", json["currency"]);
    assert!(json["type"].as_str().unwrap().starts_with("English|"));
}