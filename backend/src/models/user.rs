// src/models/user.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents the 'users' table in the database.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,

    /// Unique username.
    pub username: String,

    /// Argon2 password hash.
    /// Skipped during serialization to prevent leaking sensitive data.
    #[serde(skip)]
    pub password: String,

    /// User role: 'user' or 'admin'.
    pub role: String,

    pub created_at: Option<String>,
}

/// DTO for creating a new user (Registration).
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

/// DTO for user login.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
