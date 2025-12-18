// src/lib.rs

pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod utils;

use config::Config;
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Re-export specific items for convenience if needed
pub use routes::create_router;
