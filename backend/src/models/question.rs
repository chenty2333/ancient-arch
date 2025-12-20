// src/models/question.rs

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use validator::Validate;

/// Represents the 'questions' table in the database.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Question {
    pub id: i64,

    /// Question type: 'single' (single choice) or 'multiple' (multiple choice).
    /// Mapped from the database column 'type' since `type` is a reserved keyword in Rust.
    #[sqlx(rename = "type")]
    pub question_type: String,

    /// The text content of the question.
    pub content: String,

    /// List of options (e.g., ["Option A", "Option B"]).
    /// Stored as a JSON array in the database.
    pub options: Json<Vec<String>>,

    /// The correct answer key or content.
    pub answer: String,

    /// Explanation or analysis of the correct answer.
    pub analysis: Option<String>,

    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// DTO for sending question to client (excludes answer and analysis).
#[derive(Debug, Serialize)]
pub struct PublicQuestion {
    pub id: i64,
    #[serde(rename = "type")]
    pub question_type: String,
    pub content: String,
    pub options: Json<Vec<String>>,
}

/// DTO for creating a new question.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateQuestionRequest {
    #[validate(length(min = 1, max = 20))]
    pub question_type: String,
    #[validate(length(min = 1, max = 1000))]
    pub content: String,
    #[validate(custom(function = validate_options))]
    pub options: Vec<String>,
    #[validate(length(min = 1, max = 500))]
    pub answer: String,
    #[validate(length(max = 2000))]
    pub analysis: Option<String>,
}

fn validate_options(options: &[String]) -> Result<(), validator::ValidationError> {
    if options.is_empty() {
        return Err(validator::ValidationError::new("options_cannot_be_empty"));
    }
    for opt in options {
        if opt.len() > 500 {
            return Err(validator::ValidationError::new("option_too_long"));
        }
    }
    Ok(())
}
