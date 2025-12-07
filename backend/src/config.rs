// src/config.rs

use std::env;
use dotenvy::dotenv;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let jwt_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");
        
        let rust_log = env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string());
        
        Self {
            database_url,
            jwt_secret,
            rust_log,
        }
    }
}