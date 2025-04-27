use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use auction_site::web::app::{configure_app, init_app_state};
use log::info;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run_app(8080).await
}
