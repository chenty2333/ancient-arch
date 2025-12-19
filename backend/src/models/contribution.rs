use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Represents the 'contributions' table.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Contribution {
    pub id: i64,
    pub user_id: i64,
    pub r#type: String, // 'architecture' or 'question'
    pub data: serde_json::Value,
    pub status: String, // 'pending', 'approved', 'rejected'
    pub admin_comment: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// DTO for submission.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateContributionRequest {
    #[validate(length(min = 1))]
    pub r#type: String,
    
    /// The payload must be a valid JSON matching the target model's create request.
    pub data: serde_json::Value,
}
