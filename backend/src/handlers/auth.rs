// src/handlers/auth.rs

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    config::Config,
    error::AppError,
    models::user::{CreateUserRequest, User},
    utils::{
        hash::{hash_password, verify_password},
        jwt::sign_jwt,
    },
};

/// Registers a new user.
///
/// Hashes the password using Argon2 before storing it.
/// Returns 201 Created and the user object (excluding password).
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(validation_errors) = payload.validate() {
        return Err(AppError::BadRequest(validation_errors.to_string()));
    }

    let hashed_password = hash_password(&payload.password)?;

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, password)
        VALUES ($1, $2)
        RETURNING id, username, password, role, is_verified, created_at
        "#,
        payload.username,
        hashed_password
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        // Postgres error code for unique violation is 23505
        if e.to_string().contains("unique constraint") || e.to_string().contains("23505") {
            AppError::Conflict(format!("Username '{}' already exists", payload.username))
        } else {
            tracing::error!("Failed to register user: {:?}", e);
            AppError::from(e)
        }
    })?;

    Ok((StatusCode::CREATED, Json(user)))
}

/// Authenticates a user and returns a JWT token.
///
/// Verifies the username and password against the database.
/// If valid, signs a JWT token with the user's ID and role.
pub async fn login(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT 
            id as "id!", 
            username, 
            password, 
            role, 
            is_verified,
            created_at
        FROM users
        WHERE username = $1
        "#,
        payload.username
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Login DB error: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    let user = user.ok_or(AppError::AuthError("User not found".to_string()))?;

    let is_valid = verify_password(&payload.password, &user.password)?;

    if !is_valid {
        return Err(AppError::AuthError("Invalid password".to_string()));
    }

    let token = sign_jwt(
        user.id,
        &user.username,
        &user.role,
        &config.jwt_secret,
        config.jwt_expiration,
    )?;

    Ok(Json(json!({
        "token": token,
        "type": "Bearer",
        "is_verified": user.is_verified
    })))
}
