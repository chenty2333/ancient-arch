// src/models/exam_record.rs

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// Represents the 'exam_records' table
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ExamRecord {
    pub id: i64,
    pub user_id: i64,
    pub score: i64,
    pub created_at: Option<String>,
}

/// Aggregated struct for Leaderboard display (Joined with Users table)
#[derive(Debug, Serialize, FromRow)]
pub struct LeaderboardEntry {
    pub username: String,
    pub score: i64,
    pub created_at: Option<String>,
}

/// Request DTO for submitting a quiz
#[derive(Debug, Deserialize)]
pub struct SubmitExamRequest {
    pub answers: std::collections::HashMap<i64, String>,
}
