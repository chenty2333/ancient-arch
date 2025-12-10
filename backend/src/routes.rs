// src/routes.rs

use axum::{Router, routing::{get, post}};
use sqlx::SqlitePool;

use crate::handlers::{architecture, auth};



pub fn create_router(pool: SqlitePool) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login));
    
    let architecture_routes = Router::new()
        .route("/", get(architecture::list_architectures))
        .route("/{id}", get(architecture::get_architecture));
    
    Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/architectures", architecture_routes)
        .with_state(pool)
}