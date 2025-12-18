// src/handlers/quiz.rs

use std::collections::HashMap;

use axum::{Extension, Json, extract::State, response::IntoResponse};
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::{
        exam_record::{LeaderboardEntry, SubmitExamRequest},
        question::Question,
    },
    utils::jwt::Claims,
};

#[derive(sqlx::FromRow)]
struct AnswerKey {
    id: i64,
    answer: String,
    question_type: String,
}

// GET /api/quiz/generate
pub async fn generate_paper(State(pool): State<SqlitePool>) -> Result<impl IntoResponse, AppError> {
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
            created_at as "created_at: String"
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
            created_at as "created_at: String"
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

pub async fn submit_paper(
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<SubmitExamRequest>,
) -> Result<impl IntoResponse, AppError> {
    let question_ids: Vec<i64> = req.answers.keys().cloned().collect();

    if question_ids.is_empty() {
        return Err(AppError::BadRequest("No answers submitted".to_string()));
    }

    let mut query_builder = sqlx::QueryBuilder::new(
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
            if user_ans == &correct.answer {
                total_score += 10;
                correct_count += 1;
            }
        }
    }

    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    sqlx::query!(
        r#"
        INSERT INTO exam_records (user_id, score)
        VALUES (?, ?)
        ON CONFLICT(user_id) DO UPDATE SET
            score = CASE WHEN excluded.score > exam_records.score THEN excluded.score ELSE exam_records.score END,
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

pub async fn get_leaderboard(
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let leaderboard = sqlx::query_as!(
        LeaderboardEntry,
        r#"
        SELECT
            u.username,
            e.score,
            CAST(e.created_at AS TEXT) as "created_at: String"
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
