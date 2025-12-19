use std::collections::HashMap;

use axum::{Extension, Json, extract::State, response::IntoResponse};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres};

use crate::{
    config::{Config, EXAM_QUESTION_COUNT, PASSING_SCORE_PERCENTAGE},
    error::AppError,
    models::{
        exam_record::{ExamResponse, SubmitExamRequest},
        question::{PublicQuestion, Question},
    },
    utils::jwt::Claims as AuthClaims,
};

/// JWT Claims for the exam session to prevent tampering.
#[derive(Debug, Serialize, Deserialize)]
struct ExamClaims {
    /// List of question IDs assigned to the user.
    pub qids: Vec<i64>,
    /// Expiration timestamp.
    pub exp: usize,
}

/// Helper struct for fetching answer keys.
#[derive(sqlx::FromRow)]
struct AnswerKey {
    id: i64,
    answer: String,
}

/// Helper function to calculate score.
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
            if user_ans == correct_ans {
                correct_count += 1;
            }
        }
    }

    let score = (correct_count as f64 / total_questions as f64) * 100.0;
    (correct_count, score)
}

/// Generates a qualification exam with 20 random questions and an ExamToken.
pub async fn generate_exam(
    State(pool): State<PgPool>,
    State(config): State<Config>,
) -> Result<impl IntoResponse, AppError> {
    let questions = sqlx::query_as!(
        Question,
        r#"
        SELECT
            id, type as "question_type", content,
            options as "options: sqlx::types::Json<Vec<String>>",
            answer, analysis, created_at
        FROM questions
        ORDER BY RANDOM()
        LIMIT $1
        "#,
        EXAM_QUESTION_COUNT
    )
    .fetch_all(&pool)
    .await?;

    let qids: Vec<i64> = questions.iter().map(|q| q.id).collect();

    // Create Exam Token (Expires in 15 minutes)
    let expires_in = 900; // 15 mins
    let exp = (chrono::Utc::now().timestamp() as usize) + expires_in;
    let claims = ExamClaims { qids, exp };

    let exam_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let public_questions: Vec<PublicQuestion> = questions
        .into_iter()
        .map(|q| PublicQuestion {
            id: q.id,
            question_type: q.question_type,
            content: q.content,
            options: q.options,
        })
        .collect();

    Ok(Json(ExamResponse {
        questions: public_questions,
        exam_token,
        expires_in: expires_in as u64,
    }))
}

/// Submits the qualification exam with ExamToken verification.
pub async fn submit_exam(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<SubmitExamRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Verify Exam Token
    let token_data = decode::<ExamClaims>(
        &req.exam_token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| {
        AppError::BadRequest("Invalid or expired exam token. Please restart the exam.".to_string())
    })?;

    let allowed_qids = token_data.claims.qids;

    // 2. Security Check: Ensure user submitted exactly the questions we gave them.
    for qid in req.answers.keys() {
        if !allowed_qids.contains(qid) {
            return Err(AppError::BadRequest(format!(
                "Question ID {} was not part of this exam session.",
                qid
            )));
        }
    }

    if req.answers.len() < allowed_qids.len() {
        return Err(AppError::BadRequest(
            "Please answer all questions before submitting.".to_string(),
        ));
    }

    // 3. Fetch Answer Keys
    let mut query_builder =
        sqlx::QueryBuilder::<Postgres>::new("SELECT id, answer FROM questions WHERE id IN (");
    let mut separated = query_builder.separated(",");
    for id in &allowed_qids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    let db_answers_vec: Vec<AnswerKey> = query_builder.build_query_as().fetch_all(&pool).await?;

    let db_map: HashMap<i64, String> = db_answers_vec
        .into_iter()
        .map(|k| (k.id, k.answer))
        .collect();

    let (correct_count, score) = calculate_score(&req.answers, &db_map);
    let passed = score >= PASSING_SCORE_PERCENTAGE;
    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    if passed {
        sqlx::query!("UPDATE users SET is_verified = TRUE WHERE id = $1", user_id)
            .execute(&pool)
            .await?;
    }

    Ok(Json(serde_json::json!({
        "score": score,
        "correct_count": correct_count,
        "total_questions": db_map.len(),
        "passed": passed,
        "message": if passed { "Verification successful!" } else { "Score too low. Try again." }
    })))
}
