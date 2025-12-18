// src/main.rs

use backend::config::Config;
use backend::routes;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt};

#[tokio::main]
async fn main() {
    // Load .env file (if present)
    dotenv().ok();

    // Load configuration from environment
    let config = Config::from_env();

    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    let env_filter = EnvFilter::new(&config.rust_log);
    
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(false);
    
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);
    
    // Initialize Tracing (Logging)
    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    // Initialize Database Pool
    let pool = PgPoolOptions::new()
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
