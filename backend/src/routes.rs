// src/routes.rs

use std::sync::Arc;

use axum::{
    Router, http::Method, middleware, routing::{delete, get, post, put}
};
use sqlx::SqlitePool;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    handlers::{admin, architecture, auth, quiz},
    utils::jwt::{admin_middleware, auth_middleware},
};

/// Assembles the main application router.
///
/// * Merges all sub-routers (auth, architecture, quiz, admin).
/// * Applies global middleware (Trace, CORS).
/// * Injects global state (Database Pool).
pub fn create_router(pool: SqlitePool) -> Router {
    let origins = [
        "http://localhost:3000".parse().unwrap(),
        "http://127.0.0.1:3000".parse().unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ]);

    // let governor_conf = GovernorConfigBuilder::default()
    //     .per_second(2)
    //     .burst_size(5)
    //     .finish()
    //     .unwrap();

    // let governor_conf = Arc::new(governor_conf);

    let auth_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login));

    let architecture_routes = Router::new()
        .route("/", get(architecture::list_architectures))
        .route("/{id}", get(architecture::get_architecture));

    let quiz_routes = Router::new()
        .route("/generate", get(quiz::generate_paper))
        .route("/leaderboard", get(quiz::get_leaderboard))
        // Protected quiz routes
        .merge(
            Router::new()
                .route("/submit", post(quiz::submit_paper))
                .layer(middleware::from_fn(auth_middleware)),
        );

    let admin_routes = Router::new()
        .route("/users", get(admin::list_users).post(admin::create_user))
        .route(
            "/users/{id}",
            put(admin::update_user).delete(admin::delete_user),
        )
        .route("/architectures", post(admin::create_architecture))
        .route(
            "/architectures/{id}",
            delete(admin::delete_architecture).put(admin::update_architecture),
        )
        .route("/questions", post(admin::create_question))
        .route(
            "/questions/{id}",
            delete(admin::delete_question).put(admin::update_question),
        )
        // Double middleware protection: Auth first, then Admin check
        .layer(middleware::from_fn(admin_middleware))
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/architectures", architecture_routes)
        .nest("/api/quiz", quiz_routes)
        .nest("/api/admin", admin_routes)
        // Global Middleware (applied from outside in)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        // .layer(GovernorLayer::new(governor_conf))
        .with_state(pool)
}
