use auction_site::web::app::run_app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run_app(8080).await
}