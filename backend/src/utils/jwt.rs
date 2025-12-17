// src/utils/jwt.rs

use std::time::{SystemTime, UNIX_EPOCH};

use axum::{body::Body, http::{Request, StatusCode, header}, middleware::Next, response::Response};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{config::Config, error::AppError};


#[derive(Debug, Deserialize, Serialize, Clone)] 
pub struct Claims {
    /// Subject - the username this token belongs to
    pub sub: String,
    /// User's role
    pub role: String,
    /// Expriation time as Unix timestamp
    pub exp: usize,
}

pub fn sign_jwt(username: &str, role: &str) -> Result<String, AppError> {
    let config = Config::from_env();

    // Calculate expiration: current time + 1 hour
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .as_secs() as usize + 60 * 60;
    
    let claims = Claims {
        sub: username.to_owned(),
        role: role.to_owned(),
        exp: expiration,
    };
    
    encode(
        &Header::default(), 
        &claims, 
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(e.to_string()))
}

pub fn verify_jwt(token: &str) -> Result<Claims, AppError> {
    let config = Config::from_env();
    
    let token_data = decode(
        token, 
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()), 
        &Validation::default()
    )
    .map_err(|_| AppError::AuthError("Invalid token".to_string()))?;
    
    Ok(token_data.claims)
}

pub async fn auth_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            &header[7..]
        },
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    
    match verify_jwt(token) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        },
        Err(_) => {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub async fn admin_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = req.extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    if claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}