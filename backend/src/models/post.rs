use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Represents the 'posts' table in the database.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,

    // Using chrono for proper time handling
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,

    pub likes_count: i32,
    pub comments_count: i32,
    pub favorites_count: i32,
}

/// DTO for creating a new post.
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(
        min = 1,
        max = 200,
        message = "Title length must be between 1 and 200 chars"
    ))]
    pub title: String,

    #[validate(length(min = 1, message = "Content cannot be empty"))]
    pub content: String,
}

/// Query parameters for listing posts.
#[derive(Debug, Deserialize)]
pub struct PostListParams {
    /// Cursor for pagination: the created_at timestamp of the last post in the previous page.
    pub cursor: Option<chrono::DateTime<chrono::Utc>>,

    /// Number of items to return (default: 20, max: 100).
    pub limit: Option<i64>,
}
