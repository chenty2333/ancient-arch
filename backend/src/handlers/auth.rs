use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::user::{CresteUserRequest, User},
    utils::hash::hash_password,
};

pub async fn register(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CresteUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let hashed_password = hash_password(&payload.password)?;

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, password)
        VALUES ($1, $2)
        RETURNING id, username, password, role, created_at as "created_at: String"
        "#,
        payload.username,
        hashed_password
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            AppError::Conflict(format!("Username '{}' already exists", payload.username))
        } else {
            AppError::from(e)
        }
    })?;

    Ok((StatusCode::CREATED, Json(user)))
}
