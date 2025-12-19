// src/handlers/quiz.rs

use std::collections::HashMap;

use axum::{Extension, Json, extract::State, response::IntoResponse};
use sqlx::{PgPool, Postgres};

use crate::{
    error::AppError,
    models::{
        exam_record::{LeaderboardEntry, SubmitExamRequest},
        question::Question,
    },
    utils::jwt::Claims,
};

/// Helper struct for fetching answer keys from the database.
#[derive(sqlx::FromRow)]
struct AnswerKey {
    id: i64,
    answer: String,
    question_type: String,
}

/// Generates a random quiz paper.
///
/// Selects 6 random single-choice questions and 4 random multiple-choice questions.
/// Returns the questions without the correct answers (hidden by DTO if implemented, currently raw).
/// Note: In a production app, we should use a DTO to hide `answer` field.
pub async fn generate_paper(State(pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    let single_question = sqlx::query_as!(
        Question,
        r#"
        SELECT
            id,
            type as "question_type",
            content,
            options as "options: sqlx::types::Json<Vec<String>>",
            answer,
            analysis,
            created_at::TEXT as "created_at: String"
        FROM questions
        WHERE type = 'single'
        ORDER BY RANDOM()
        LIMIT 6
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch single question: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    let multiple_questions = sqlx::query_as!(
        Question,
        r#"
        SELECT
            id,
            type as "question_type",
            content,
            options as "options: sqlx::types::Json<Vec<String>>",
            answer,
            analysis,
            created_at::TEXT as "created_at: String"
        FROM questions
        WHERE type = 'multiple'
        ORDER BY RANDOM()
        LIMIT 4
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch multiple questions: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    let mut paper = Vec::new();
    paper.extend(single_question);
    paper.extend(multiple_questions);

    Ok(Json(paper))
}

/// Submits a user's exam answers and calculates the score.
///
/// * Validates the token and extracts User ID.
/// * Compares user answers with database records.
/// * Calculates score (10 points per correct answer).
/// * Saves or updates the result (Upsert) in `exam_records`.
pub async fn submit_paper(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<SubmitExamRequest>,
) -> Result<impl IntoResponse, AppError> {
    let question_ids: Vec<i64> = req.answers.keys().cloned().collect();

    if question_ids.is_empty() {
        return Err(AppError::BadRequest("No answers submitted".to_string()));
    }

    // Use QueryBuilder for dynamic IN clause
    let mut query_builder = sqlx::QueryBuilder::<Postgres>::new(
        "SELECT
            id,
            answer,
            type as question_type FROM questions WHERE id IN (",
    );

    let mut separated = query_builder.separated(",");
    for id in &question_ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    let db_answers: Vec<AnswerKey> = query_builder
        .build_query_as()
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut total_score = 0;
    let mut correct_count = 0;

    let db_map: HashMap<i64, AnswerKey> = db_answers.into_iter().map(|k| (k.id, k)).collect();

    for (q_id, user_ans) in &req.answers {
        if let Some(correct) = db_map.get(q_id) {
            // Simple strict string matching
            if user_ans == &correct.answer {
                total_score += 10;
                correct_count += 1;
            }
        }
    }

    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    // Upsert: keep the highest score if user retakes the exam
    sqlx::query!(
        r#"
        INSERT INTO exam_records (user_id, score)
        VALUES ($1, $2)
        ON CONFLICT(user_id) DO UPDATE SET
            score = CASE WHEN EXCLUDED.score > exam_records.score THEN EXCLUDED.score ELSE exam_records.score END,
            created_at = CURRENT_TIMESTAMP
        "#,
        user_id,
        total_score
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to upsert exam record: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    Ok(Json(serde_json::json!({
        "score": total_score,
        "correct_count": correct_count,
        "total_questions": question_ids.len(),
        "message": "Exam submmited successfully"
    })))
}

/// Retrieves the top 5 high scores from the leaderboard.
pub async fn get_leaderboard(State(pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    let leaderboard = sqlx::query_as!(
        LeaderboardEntry,
        r#"
        SELECT
            u.username,
            e.score,
            e.created_at::TEXT as "created_at: String"
        FROM exam_records e
        JOIN users u ON e.user_id = u.id
        ORDER BY e.score DESC
        LIMIT 5
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch leaderboard: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    Ok(Json(leaderboard))
}
