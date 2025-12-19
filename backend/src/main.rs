// src/main.rs

use backend::config::Config;
use backend::routes;
use backend::state::AppState;
use backend::utils::hash::hash_password;
use dotenvy::dotenv;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::time::Duration;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load .env file (if present)
    dotenv().ok();

    // Load configuration from environment
    let config = Config::from_env();

    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let env_filter = EnvFilter::new(&config.rust_log);
    let stdout_layer = fmt::layer().with_writer(std::io::stdout).with_target(false);
    let file_layer = fmt::layer().with_writer(non_blocking).with_ansi(false);

    // Initialize Tracing (Logging)
    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    // Initialize Database Pool with Retry
    let mut retry_count = 0;
    let pool = loop {
        match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&config.database_url)
            .await
        {
            Ok(pool) => break pool,
            Err(e) => {
                retry_count += 1;
                if retry_count > 5 {
                    panic!("Failed to connect to database after 5 retries: {}", e);
                }
                tracing::warn!("Database not ready, retrying in 2s... (Attempt {})", retry_count);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    };

    tracing::info!("Database connected...");

    // Run Migrations Automatically
    tracing::info!("Running migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    tracing::info!("Migrations applied successfully.");

    // Seed Admin User
    if let Err(e) = seed_admin_user(&pool, &config).await {
        tracing::error!("Failed to seed admin user: {:?}", e);
    }

    // Create AppState
    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    // Create the Axum application router
    let app = routes::create_router(state);

    // Bind to the listening address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("zk Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // Start the server
    axum::serve(listener, app).await.unwrap();
}

async fn seed_admin_user(pool: &PgPool, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if let (Some(username), Some(password)) = (&config.admin_username, &config.admin_password) {
        let user_exists = sqlx::query!("SELECT id FROM users WHERE username = $1", username)
            .fetch_optional(pool)
            .await?;

        if user_exists.is_none() {
            tracing::info!("Seeding admin user: {}", username);
            let hashed_password = hash_password(password)?;

            sqlx::query!(
                "INSERT INTO users (username, password, role) VALUES ($1, $2, 'admin')",
                username,
                hashed_password
            )
            .execute(pool)
            .await?;
            tracing::info!("Admin user created successfully.");
        }
    }
    Ok(())
}