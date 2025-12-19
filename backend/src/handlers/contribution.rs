use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        contribution::CreateContributionRequest,
        architecture::CreateArchRequest,
        question::CreateQuestionRequest,
    },
    utils::jwt::VerifiedUser,
};

/// Submit a new contribution.
/// Enforces "once per day" via DB index and strict data validation.
pub async fn create_contribution(
    State(pool): State<PgPool>,
    user: VerifiedUser,
    Json(payload): Json<CreateContributionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Basic validation
    payload.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    // 2. Strict Payload Validation
    if payload.r#type != "architecture" && payload.r#type != "question" {
        return Err(AppError::BadRequest("Invalid contribution type".to_string()));
    }

    // We try to deserialize the JSON 'data' to ensure it's valid for the target type.
    match payload.r#type.as_str() {
        "architecture" => {
            let _: CreateArchRequest = serde_json::from_value(payload.data.clone())
                .map_err(|e| AppError::BadRequest(format!("Invalid architecture data: {}", e)))?;
        },
        "question" => {
            let _: CreateQuestionRequest = serde_json::from_value(payload.data.clone())
                .map_err(|e| AppError::BadRequest(format!("Invalid question data: {}", e)))?;
        },
        _ => unreachable!(), // Handled by validator
    }

    // 3. Insert into DB
    let id = sqlx::query!(
        r#"
        INSERT INTO contributions (user_id, type, data)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        user.id,
        payload.r#type,
        payload.data
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        // Handle "once per day" unique constraint violation
        if e.to_string().contains("idx_user_daily_contribution") {
            AppError::Conflict("You have already submitted a contribution today. Please try again tomorrow.".to_string())
        } else {
            tracing::error!("Failed to submit contribution: {:?}", e);
            AppError::InternalServerError(e.to_string())
        }
    })?
    .id;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}
