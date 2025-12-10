// src/main.rs

mod config;
pub mod models;
pub mod utils;
pub mod error;
pub mod handlers;
pub mod routes;

use config::Config;
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // .ok() ignores errors - it's fine if .env file doesn't exist
    dotenv().ok();
    let config = Config::from_env();
    
    // No longer create the default layer via tracing_subscriber::fmt()
    // Instead, use registry() for flexibility to add more layers later
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    tracing::info!("Database connected...");
    
    
    let app = routes::create_router(pool);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("zk Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    axum::serve(listener, app)
        .await
        .unwrap();
}

// async fn health_check() -> &'static str {
//     "OK"
// }
