// src/models/question.rs

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Question {
    pub id: i64,

    #[sqlx(rename = "type")]
    pub question_type: String,

    pub content: String,

    pub options: Json<Vec<String>>,

    pub answer: String,

    pub analysis: Option<String>,

    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuestionRequest {
    pub question_type: String,
    pub content: String,
    pub options: Vec<String>,
    pub answer: String,
    pub analysis: Option<String>,
}
