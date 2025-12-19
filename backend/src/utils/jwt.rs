use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::Body,
    extract::{FromRef, FromRequestParts, State},
    http::{Request, StatusCode, header, request::Parts},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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

/// A custom extractor that only allows verified users or admins.
pub struct VerifiedUser {
    pub id: i64,
}

impl<S> FromRequestParts<S> for VerifiedUser
where
    PgPool: FromRef<S>,
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Get dependencies from state
        let pool = PgPool::from_ref(state);
        let config = Config::from_ref(state);

        // 2. Extract and verify Token
        let claims = extract_claims_from_header(&parts.headers, &config.jwt_secret)
            .ok_or(AppError::AuthError("Missing or invalid token".to_string()))?;

        let user_id = claims.sub.parse::<i64>().unwrap_or(0);

        // 3. Check DB status
        let user = sqlx::query!("SELECT is_verified, role FROM users WHERE id = $1", user_id)
            .fetch_optional(&pool)
            .await?
            .ok_or(AppError::NotFound("User not found".to_string()))?;

        if user.is_verified || user.role == "admin" {
            Ok(VerifiedUser { id: user_id })
        } else {
            Err(AppError::AuthError(
                "You must be a verified contributor to perform this action.".to_string(),
            ))
        }
    }
}

/// Signs a new JWT for the user.
pub fn sign_jwt(
    id: i64,
    _username: &str,
    role: &str,
    secret: &str,
    expiration_seconds: u64,
) -> Result<String, AppError> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .as_secs() as usize
        + expiration_seconds as usize;

    let claims = Claims {
        sub: id.to_string(),
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

/// Helper to extract and verify JWT from Authorization header.
pub fn extract_claims_from_header(headers: &header::HeaderMap, secret: &str) -> Option<Claims> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .filter(|auth| auth.starts_with("Bearer "))
        .and_then(|auth| verify_jwt(&auth[7..], secret).ok())
}

/// Verifies and decodes a JWT string.
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let token_data = decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::AuthError("Invalid token".to_string()))?;

    Ok(token_data.claims)
}

/// Mandatory Authentication Middleware.
pub async fn auth_middleware(
    State(config): State<Config>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(claims) = extract_claims_from_header(req.headers(), &config.jwt_secret) {
        req.extensions_mut().insert(claims);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// Optional Authentication Middleware.
pub async fn optional_auth_middleware(
    State(config): State<Config>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(claims) = extract_claims_from_header(req.headers(), &config.jwt_secret) {
        req.extensions_mut().insert(claims);
    }
    Ok(next.run(req).await)
}

/// Admin Authorization Middleware (Must follow auth_middleware).
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
