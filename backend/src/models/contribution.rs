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
    #[validate(length(min = 1, max = 20), custom(function = validate_contribution_type))]
    pub r#type: String,

    /// The payload must be a valid JSON matching the target model's create request.
    #[validate(custom(function = validate_data_size))]
    pub data: serde_json::Value,
}

/// Restricts the contribution type to 'architecture' or 'question'.
fn validate_contribution_type(c_type: &str) -> Result<(), validator::ValidationError> {
    if c_type != "architecture" && c_type != "question" {
        return Err(validator::ValidationError::new("invalid_contribution_type"));
    }
    Ok(())
}

/// Limits the total JSON payload size to prevent resource exhaustion attacks.
fn validate_data_size(data: &serde_json::Value) -> Result<(), validator::ValidationError> {
    // Limit total JSON size to roughly 50KB to prevent abuse
    if data.to_string().len() > 50000 {
        return Err(validator::ValidationError::new("payload_too_large"));
    }
    Ok(())
}
