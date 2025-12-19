use std::collections::HashMap;

use axum::{Extension, Json, extract::State, response::IntoResponse};
use sqlx::{PgPool, Postgres};

use crate::{
    config::{EXAM_QUESTION_COUNT, PASSING_SCORE_PERCENTAGE},
    error::AppError,
    models::{
        exam_record::SubmitExamRequest,
        question::{PublicQuestion, Question},
    },
    utils::jwt::Claims,
};

/// Helper struct for fetching answer keys.
#[derive(sqlx::FromRow)]
struct AnswerKey {
    id: i64,
    answer: String,
}

/// Helper function to calculate score.
/// Returns (correct_count, score_percentage).
fn calculate_score(
    user_answers: &HashMap<i64, String>,
    db_answers: &HashMap<i64, String>,
) -> (usize, f64) {
    let mut correct_count = 0;
    let total_questions = db_answers.len();

    if total_questions == 0 {
        return (0, 0.0);
    }

    for (q_id, user_ans) in user_answers {
        if let Some(correct_ans) = db_answers.get(q_id) {
            // Case-insensitive comparison could be added here if needed,
            // but strict matching is safer for now.
            if user_ans == correct_ans {
                correct_count += 1;
            }
        }
    }

    let score = (correct_count as f64 / total_questions as f64) * 100.0;
    (correct_count, score)
}

/// Generates a qualification exam with 20 random questions.
pub async fn generate_exam(State(pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    // Fetch 20 random questions.
    // We try to mix types if possible, but for simplicity we take 20 random.
    let questions = sqlx::query_as!(
        Question,
        r#"
        SELECT
            id,
            type as "question_type",
            content,
            options as "options: sqlx::types::Json<Vec<String>>",
            answer,
            analysis,
            created_at
        FROM questions
        ORDER BY RANDOM()
        LIMIT $1
        "#,
        EXAM_QUESTION_COUNT
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch qualification questions: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    // Map to PublicQuestion to hide sensitive data
    let public_questions: Vec<PublicQuestion> = questions
        .into_iter()
        .map(|q| PublicQuestion {
            id: q.id,
            question_type: q.question_type,
            content: q.content,
            options: q.options,
        })
        .collect();

    Ok(Json(public_questions))
}

/// Submits the qualification exam.
/// If score >= 60%, updates `users.is_verified = TRUE`.
pub async fn submit_exam(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<SubmitExamRequest>,
) -> Result<impl IntoResponse, AppError> {
    let question_ids: Vec<i64> = req.answers.keys().cloned().collect();

    if question_ids.is_empty() {
        return Err(AppError::BadRequest("No answers submitted".to_string()));
    }

    // Dynamic IN clause to fetch answers
    let mut query_builder =
        sqlx::QueryBuilder::<Postgres>::new("SELECT id, answer FROM questions WHERE id IN (");

    let mut separated = query_builder.separated(",");
    for id in &question_ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    let db_answers_vec: Vec<AnswerKey> = query_builder
        .build_query_as()
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let db_map: HashMap<i64, String> = db_answers_vec
        .into_iter()
        .map(|k| (k.id, k.answer))
        .collect();

    let (correct_count, score) = calculate_score(&req.answers, &db_map);
    let passed = score >= PASSING_SCORE_PERCENTAGE;
    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    if passed {
        // Update user verification status
        sqlx::query!("UPDATE users SET is_verified = TRUE WHERE id = $1", user_id)
            .execute(&pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update user verification: {:?}", e);
                AppError::InternalServerError(e.to_string())
            })?;
    }

    Ok(Json(serde_json::json!({
        "score": score,
        "correct_count": correct_count,
        "total_questions": db_map.len(),
        "passed": passed,
        "message": if passed { "Verification successful!" } else { "Score too low. Try again." }
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_score_perfect() {
        let mut user_answers = HashMap::new();
        user_answers.insert(1, "A".to_string());
        user_answers.insert(2, "B".to_string());

        let mut db_answers = HashMap::new();
        db_answers.insert(1, "A".to_string());
        db_answers.insert(2, "B".to_string());

        let (correct, score) = calculate_score(&user_answers, &db_answers);
        assert_eq!(correct, 2);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_calculate_score_half() {
        let mut user_answers = HashMap::new();
        user_answers.insert(1, "A".to_string());
        user_answers.insert(2, "C".to_string()); // Wrong

        let mut db_answers = HashMap::new();
        db_answers.insert(1, "A".to_string());
        db_answers.insert(2, "B".to_string());

        let (correct, score) = calculate_score(&user_answers, &db_answers);
        assert_eq!(correct, 1);
        assert_eq!(score, 50.0);
    }

    #[test]
    fn test_calculate_score_pass_threshold() {
        // 5 questions. Need 3 correct for 60%.
        let mut db_answers = HashMap::new();
        for i in 1..=5 {
            db_answers.insert(i, "A".to_string());
        }

        let mut user_answers = HashMap::new();
        user_answers.insert(1, "A".to_string());
        user_answers.insert(2, "A".to_string());
        user_answers.insert(3, "A".to_string());
        user_answers.insert(4, "B".to_string()); // Wrong
        user_answers.insert(5, "B".to_string()); // Wrong

        let (correct, score) = calculate_score(&user_answers, &db_answers);
        assert_eq!(correct, 3);
        assert_eq!(score, 60.0);
    }

    #[test]
    fn test_calculate_score_zero() {
        let mut user_answers = HashMap::new();
        user_answers.insert(1, "B".to_string());

        let mut db_answers = HashMap::new();
        db_answers.insert(1, "A".to_string());

        let (correct, score) = calculate_score(&user_answers, &db_answers);
        assert_eq!(correct, 0);
        assert_eq!(score, 0.0);
    }
}
