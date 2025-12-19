use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Represents the 'comments' table in the database.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub user_id: i64,
    pub content: String,
    pub root_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// DTO for creating a new comment.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCommentRequest {
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Comment must be between 1 and 1000 characters"
    ))]
    pub content: String,

    /// Optional: the ID of the comment being replied to.
    pub parent_id: Option<i64>,
}

/// DTO for displaying a comment with author info.
#[derive(Debug, Serialize, FromRow)]
pub struct CommentResponse {
    pub id: i64,
    pub post_id: i64,
    pub user_id: i64,
    pub username: String,
    pub content: String,
    pub root_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
