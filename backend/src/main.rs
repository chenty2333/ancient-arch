// src/main.rs

mod config;
use config::Config;
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    
    tracing_subscriber::fmt()
        .with_env_filter(&config.rust_log)
        .init();
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    tracing::info!("Database connected...");
    
    let app = Router::new()
        .route("/health", get(health_check))
        .with_state(pool);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("zk Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}
