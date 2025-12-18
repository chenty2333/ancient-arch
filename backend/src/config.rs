// src/config.rs

use dotenvy::dotenv;
use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Database connection string (SQLite).
    pub database_url: String,
    /// Secret key for signing JWTs.
    pub jwt_secret: String,
    /// Logging level (e.g., "info", "debug").
    pub rust_log: String,
}

impl Config {
    /// Loads configuration from `.env` file and environment variables.
    /// Panics if required variables are missing.
    pub fn from_env() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Self {
            database_url,
            jwt_secret,
            rust_log,
        }
    }
}
