use axum::{Router, routing::post};
use sqlx::SqlitePool;

use crate::handlers::auth;



pub fn create_router(pool: SqlitePool) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login));
    
    Router::new()
        .nest("/api/auth", auth_routes)
        .with_state(pool)
}