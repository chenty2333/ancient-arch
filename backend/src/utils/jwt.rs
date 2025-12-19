// src/utils/jwt.rs

use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{config::Config, error::AppError};

/// JWT Claims structure.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Claims {
    /// Subject - Stores the User ID (as string).
    pub sub: String,
    /// User's role (e.g., 'user', 'admin').
    pub role: String,
    /// Expiration time as Unix timestamp.
    pub exp: usize,
}

/// Signs a new JWT for the user.
///
/// Arguments:
/// * `id`: User ID.
/// * `role`: User role.
pub fn sign_jwt(
    id: i64,
    _username: &str,
    role: &str,
    secret: &str,
    expiration_seconds: u64,
) -> Result<String, AppError> {
    // Calculate expiration: current time + expiration_seconds
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .as_secs() as usize
        + expiration_seconds as usize;

    let claims = Claims {
        sub: id.to_string(), // Store User ID in 'sub' claim
        role: role.to_owned(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(e.to_string()))
}

/// Verifies and decodes a JWT string.
///
/// Returns the `Claims` if valid, otherwise returns an `AppError`.
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let token_data = decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::AuthError("Invalid token".to_string()))?;

    Ok(token_data.claims)
}

/// Axum Middleware: Authentication.
///
/// Intercepts requests, validates the 'Authorization: Bearer <token>' header.
/// If valid, injects `Claims` into the request extensions for handlers to use.
/// If invalid, returns 401 Unauthorized.
pub async fn auth_middleware(
    State(config): State<Config>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    match verify_jwt(token, &config.jwt_secret) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Axum Middleware: Admin Authorization.
///
/// Must be used AFTER `auth_middleware`. Checks if the injected `Claims` has 'admin' role.
/// If not, returns 403 Forbidden.
pub async fn admin_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}
