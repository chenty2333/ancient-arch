// src/lib.rs

pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod state;
pub mod utils;

// Re-export specific items for convenience if needed
pub use routes::create_router;
