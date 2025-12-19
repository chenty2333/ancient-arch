// src/routes.rs

// use std::sync::Arc;

use axum::{
    Router,
    http::Method,
    middleware,
    routing::{delete, get, post, put},
};
// use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    handlers::{
        admin, architecture, auth, community, contribution, interaction, profile, qualification,
        quiz,
    },
    state::AppState,
    utils::jwt::{admin_middleware, auth_middleware, optional_auth_middleware},
};

/// Assembles the main application router.
///
/// * Merges all sub-routers (auth, architecture, quiz, admin).
/// * Applies global middleware (Trace, CORS).
/// * Injects global state (Database Pool).
pub fn create_router(state: AppState) -> Router {
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
        .route("/login", post(auth::login))
        // Qualification routes (Protected)
        .merge(
            Router::new()
                .route("/qualification", get(qualification::generate_exam))
                .route("/qualification/submit", post(qualification::submit_exam))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                )),
        );

    let architecture_routes = Router::new()
        .route("/", get(architecture::list_architectures))
        .route("/{id}", get(architecture::get_architecture));

    let post_routes = Router::new()
        .route("/", get(community::list_posts))
        .route(
            "/{id}",
            get(community::get_post).layer(middleware::from_fn_with_state(
                state.clone(),
                optional_auth_middleware,
            )),
        )
        .route("/{id}/comments", get(interaction::list_comments))
        .merge(
            Router::new()
                .route("/", post(community::create_post))
                .route("/{id}", delete(community::delete_post))
                .route("/{id}/like", post(interaction::toggle_like))
                .route("/{id}/favorite", post(interaction::toggle_favorite))
                .route("/{id}/comments", post(interaction::create_comment))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                )),
        );

    let profile_routes = Router::new()
        .route("/me", get(profile::get_me))
        .route("/posts", get(profile::list_my_posts))
        .route("/favorites", get(profile::list_my_favorites))
        .route("/contributions", get(profile::list_my_contributions))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let contribution_routes = Router::new()
        .route("/", post(contribution::create_contribution))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let quiz_routes = Router::new()
        .route("/generate", get(quiz::generate_paper))
        .route("/leaderboard", get(quiz::get_leaderboard))
        // Protected quiz routes
        .merge(
            Router::new()
                .route("/submit", post(quiz::submit_paper))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                )),
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
        .route("/contributions", get(admin::list_contributions))
        .route(
            "/contributions/{id}/review",
            put(admin::review_contribution),
        )
        // Double middleware protection: Auth first, then Admin check
        .layer(middleware::from_fn(admin_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/architectures", architecture_routes)
        .nest("/api/posts", post_routes)
        .nest("/api/profile", profile_routes)
        .nest("/api/contributions", contribution_routes)
        .nest("/api/quiz", quiz_routes)
        .nest("/api/admin", admin_routes)
        // Global Middleware (applied from outside in)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        // .layer(GovernorLayer::new(governor_conf))
        .with_state(state)
}
