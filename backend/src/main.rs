// src/main.rs

mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod utils;

use config::Config;
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load .env file (if present)
    dotenv().ok();

    // Load configuration from environment
    let config = Config::from_env();

    // Initialize Tracing (Logging)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize Database Pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Database connected...");

    // Create the Axum application router
    let app = routes::create_router(pool);

    // Bind to the listening address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("zk Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // Start the server
    axum::serve(listener, app).await.unwrap();
}

// async fn health_check() -> &'static str {                                                        │
//     "OK"
// }
