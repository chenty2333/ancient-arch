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

    /// Whether the user has passed the qualification exam.
    pub is_verified: bool,

    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Aggregated user profile data for the current user.
#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub is_verified: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub posts_count: i64,
    pub total_likes_received: i64,
}

/// DTO for a favorited post item, including joined post info.
#[derive(Debug, Serialize, FromRow)]
pub struct FavoritePostResponse {
    pub post_id: i64,
    pub title: String,
    pub author_username: String,
    pub favorited_at: chrono::DateTime<chrono::Utc>,
}

/// DTO for creating a new user (Registration).
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username length must be between 3 and 50 characters."
    ))]
    pub username: String,
    #[validate(length(
        min = 4,
        max = 128,
        message = "Password length must be between 4 and 128 characters."
    ))]
    pub password: String,
}

/// DTO for user login.
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, max = 50))]
    pub username: String,
    #[validate(length(min = 1, max = 128))]
    pub password: String,
}
