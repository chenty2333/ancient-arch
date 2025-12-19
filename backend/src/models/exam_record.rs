// src/models/exam_record.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents the 'exam_records' table in the database.
/// Stores the results of user quizzes.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ExamRecord {
    pub id: i64,
    pub user_id: i64,
    pub score: i64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Aggregated struct for displaying the leaderboard.
/// Represents a row joined from `users` and `exam_records`.
#[derive(Debug, Serialize, FromRow)]
pub struct LeaderboardEntry {
    pub username: String,
    pub score: i64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// DTO for submitting a quiz attempt.
#[derive(Debug, Deserialize)]
pub struct SubmitExamRequest {
    /// User's answers map.
    /// Key: Question ID (i64)
    /// Value: User's selected option (String)
    pub answers: std::collections::HashMap<i64, String>,
}
