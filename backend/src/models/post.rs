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

    /// UI helper: whether the current user has liked this post.
    /// Default to false, populated only in specific queries.
    #[serde(default)]
    pub is_liked: bool,
    /// UI helper: whether the current user has favorited this post.
    #[serde(default)]
    pub is_favorited: bool,
}

/// DTO for creating a new post.
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Title length must be between 1 and 100 chars"
    ))]
    pub title: String,

    #[validate(length(
        min = 1,
        max = 10000,
        message = "Content length must be between 1 and 10000 chars"
    ))]
    pub content: String,
}

/// Query parameters for listing posts.
#[derive(Debug, Deserialize)]
pub struct PostListParams {
    /// Cursor for pagination: the created_at timestamp of the last post in the previous page.
    pub cursor: Option<chrono::DateTime<chrono::Utc>>,

    /// Number of items to return (default: 20, max: 100).
    pub limit: Option<i64>,

    /// Sort order: 'new' (default) or 'hot'.
    pub sort: Option<String>,

    /// Search keyword for title match.
    pub q: Option<String>,
}
