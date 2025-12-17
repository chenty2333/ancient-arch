// src/routes.rs

use axum::{
    Router, middleware,
    routing::{get, post},
};
use sqlx::SqlitePool;

use crate::{
    handlers::{architecture, auth, quiz},
    utils::jwt::auth_middleware,
};

pub fn create_router(pool: SqlitePool) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login));

        let architecture_routes = Router::new()
            .route("/", get(architecture::list_architectures))
            .route("/{id}", get(architecture::get_architecture));
        
        let quiz_routes = Router::new()
            .route("/generate", get(quiz::generate_paper))
            .route("/leaderboard", get(quiz::get_leaderboard))
            .merge(
                Router::new()
                    .route("/submit", post(quiz::submit_paper))
                    .layer(middleware::from_fn(auth_middleware))
            );
        
        Router::new()
            .nest("/api/auth", auth_routes)
            .nest("/api/architectures", architecture_routes)
            .nest("/api/quiz", quiz_routes)
            .with_state(pool)
    }
