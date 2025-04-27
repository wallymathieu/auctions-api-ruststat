use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web::middleware::Logger;
use base64::{Engine as _, engine::general_purpose};
use log::info;
use serde_json::Value;
use time::OffsetDateTime;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::domain::{auctions, AuctionId, Bid, Command, User, handle};
use crate::domain::states::{get_bids, try_get_amount_and_winner};
use crate::money::Amount;
use super::types::{AddAuctionRequest, ApiError, AppState, AuctionBid, AuctionDetail, AuctionItem, BidRequest};

// Initialize application state
pub fn init_app_state() -> AppState {
    Arc::new(Mutex::new(HashMap::new()))
}

// Read x-jwt-payload header and extract user information
fn get_auth_user(req: &HttpRequest) -> Option<User> {
    let auth_header = req.headers().get("x-jwt-payload")?;
    let auth_str = auth_header.to_str().ok()?;
    
    // Decode base64
    let decoded = general_purpose::STANDARD.decode(auth_str).ok()?;
    let json_str = String::from_utf8(decoded).ok()?;
    
    // Parse JSON
    let json: Value = serde_json::from_str(&json_str).ok()?;
    
    // Extract user fields
    let sub = json.get("sub")?.as_str()?;
    let u_typ = json.get("u_typ")?.as_str()?;
    
    if u_typ == "0" {
        let name = json.get("name")?.as_str()?;
        Some(User::BuyerOrSeller {
            user_id: sub.to_string(),
            name: name.to_string(),
        })
    } else if u_typ == "1" {
        Some(User::Support {
            user_id: sub.to_string(),
        })
    } else {
        None
    }
}

// Middleware to require authentication
async fn with_auth<F>(req: HttpRequest, f: F) -> Result<HttpResponse>
where
    F: FnOnce(User) -> Result<HttpResponse>
{
    match get_auth_user(&req) {
        Some(user) => {
            let result = f(user)?;
            Ok(result)
        },
        None => {
            Ok(HttpResponse::Unauthorized().body("Unauthorized"))
        }
    }
}

// Get all auctions
async fn get_auctions(data: web::Data<AppState>) -> Result<HttpResponse> {
    let app_state = data.lock().unwrap();
    let auction_list: Vec<AuctionItem> = auctions(&app_state)
        .iter()
        .map(|a| AuctionItem::from(a))
        .collect();
    
    Ok(HttpResponse::Ok().json(auction_list))
}

// Get auction by ID
async fn get_auction(
    path: web::Path<AuctionId>,
    data: web::Data<AppState>
) -> Result<HttpResponse> {
    let auction_id = path.into_inner();
    let app_state = data.lock().unwrap();
    
    if let Some((auction, auction_state)) = app_state.get(&auction_id) {
        let bids = get_bids(auction_state);
        let winner_and_price = try_get_amount_and_winner(auction_state);
        
        let auction_bids = bids.iter().map(|bid| {
            AuctionBid {
                amount: bid.bid_amount,
                bidder: bid.bidder.clone(),
            }
        }).collect();
        
        let (winner, winner_price) = match winner_and_price {
            Some((amount, user_id)) => (Some(user_id), Some(amount)),
            None => (None, None),
        };
        
        let detail = AuctionDetail {
            id: auction.auction_id,
            starts_at: auction.starts_at,
            title: auction.title.clone(),
            expiry: auction.expiry,
            currency: auction.auction_currency,
            bids: auction_bids,
            winner,
            winner_price: winner_price.map(|v| Amount::new(auction.auction_currency, v)),
        };
        
        Ok(HttpResponse::Ok().json(detail))
    } else {
        let error = ApiError {
            message: "Auction not found".to_string(),
        };
        Ok(HttpResponse::NotFound().json(error))
    }
}

// Create a new auction
async fn create_auction(
    req: HttpRequest,
    auction_req: web::Json<AddAuctionRequest>,
    data: web::Data<AppState>
) -> Result<HttpResponse> {
    with_auth(req, |user| {
        let auction = auction_req.to_auction(user);
        let now = OffsetDateTime::now_utc();
        let command = Command::AddAuction {
            timestamp: now,
            auction: auction.clone(),
        };
        
        let mut app_state = data.lock().unwrap();
        
        match handle(command, app_state.clone()) {
            Ok((success, new_state)) => {
                *app_state = new_state;
                Ok(HttpResponse::Ok().json(success))
            },
            Err(err) => {
                Ok(HttpResponse::BadRequest().body(format!("{}", err)))
            }
        }
    }).await
}

// Place a bid on an auction
async fn place_bid(
    req: HttpRequest,
    path: web::Path<AuctionId>,
    bid_req: web::Json<BidRequest>,
    data: web::Data<AppState>
) -> Result<HttpResponse> {
    let auction_id = path.into_inner();
    
    with_auth(req, |user| {
        let now = OffsetDateTime::now_utc();
        
        let bid = Bid {
            for_auction: auction_id,
            bidder: user,
            at: now,
            bid_amount: bid_req.amount,
        };
        
        let command = Command::PlaceBid {
            timestamp: now,
            bid,
        };
        
        let mut app_state = data.lock().unwrap();
        
        match handle(command, app_state.clone()) {
            Ok((success, new_state)) => {
                *app_state = new_state;
                Ok(HttpResponse::Ok().json(success))
            },
            Err(err) => {
                Ok(HttpResponse::BadRequest().body(format!("{}", err)))
            }
        }
    }).await
}

// Configure routes
pub fn configure_app(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/auctions", web::get().to(get_auctions))
            .route("/auctions/{id}", web::get().to(get_auction))
            .route("/auctions", web::post().to(create_auction))
            .route("/auctions/{id}/bids", web::post().to(place_bid))
    );
}

// Main application
pub async fn run_app(port: u16) -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    
    let app_state = init_app_state();
    
    info!("Starting server on port {}", port);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .configure(configure_app)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}