// src/routes.rs

use axum::{
    Router, middleware,
    routing::{delete, get, post},
};
use sqlx::SqlitePool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    handlers::{admin, architecture, auth, quiz},
    utils::jwt::{admin_middleware, auth_middleware},
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
                .layer(middleware::from_fn(auth_middleware)),
        );

    let admin_routes = Router::new()
        .route("/users", get(admin::list_users))
        .route("/architectures", post(admin::create_architecture))
        .route("/architectures/{id}", delete(admin::delete_architecture))
        .layer(middleware::from_fn(admin_middleware))
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/architectures", architecture_routes)
        .nest("/api/quiz", quiz_routes)
        .nest("/api/admin", admin_routes)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(pool)
}
