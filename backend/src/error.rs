// src/error.rs

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::fmt;

/// Global Application Error Enum.
/// Centralizes error handling and mapping to HTTP responses.
#[derive(Debug)]
pub enum AppError {
    // 500 Internal Server Error
    InternalServerError(String),

    // 400 Bad Request
    BadRequest(String),

    // 401 Unauthorized
    AuthError(String),

    // 404 Not Found
    NotFound(String),

    // 409 Conflict (e.g., duplicate username)
    Conflict(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AppError {}

/// Implements `IntoResponse` for `AppError`.
/// Converts the error into a JSON response with appropriate HTTP status code.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError(msg) => {
                tracing::error!("Internal Server Error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
        };
        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Converts `sqlx::Error` into `AppError::InternalServerError`.
/// Allows using `?` operator on database queries.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::BadRequest(err.to_string())
    }
}
