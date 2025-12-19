// src/models/user.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

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
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Username length must be between 3 and 20 characters."
    ))]
    pub username: String,
    #[validate(length(
        min = 4,
        max = 20,
        message = "Password length must be between 4 and 20 characters."
    ))]
    pub password: String,
}

/// DTO for user login.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
