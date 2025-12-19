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
    /// JWT expiration time in seconds (default: 3600).
    pub jwt_expiration: u64,
    /// Admin username for initial seeding.
    pub admin_username: Option<String>,
    /// Admin password for initial seeding.
    pub admin_password: Option<String>,
}

// Business Logic Constants
pub const EXAM_QUESTION_COUNT: i64 = 20;
pub const PASSING_SCORE_PERCENTAGE: f64 = 60.0;

impl Config {
    /// Loads configuration from `.env` file and environment variables.
    /// Panics if required variables are missing.
    pub fn from_env() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        let jwt_expiration = env::var("JWT_EXPIRATION")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .expect("JWT_EXPIRATION must be a number");

        let admin_username = env::var("ADMIN_USERNAME").ok();
        let admin_password = env::var("ADMIN_PASSWORD").ok();

        Self {
            database_url,
            jwt_secret,
            rust_log,
            jwt_expiration,
            admin_username,
            admin_password,
        }
    }
}
