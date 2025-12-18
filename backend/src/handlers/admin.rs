// src/handlers/admin.rs

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{error::AppError, models::user::User};

/// Lists all users in the system.
/// Admin only.
pub async fn list_users(State(pool): State<SqlitePool>) -> Result<impl IntoResponse, AppError> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT
        id, username, password, role,
        created_at as "created_at: String"
        FROM users
        ORDER BY id DESC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list users: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    Ok(Json(users))
}

/// DTO for creating a new architecture entry.
#[derive(Debug, Deserialize)]
pub struct CreateArchRequest {
    pub category: String,
    pub name: String,
    pub dynasty: String,
    pub location: String,
    pub description: String,
    pub cover_img: String,
    pub carousel_imgs: Vec<String>,
}

/// Creates a new architecture entry.
/// Admin only.
pub async fn create_architecture(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateArchRequest>,
) -> Result<impl IntoResponse, AppError> {
    let carousel_imgs_json = sqlx::types::Json(payload.carousel_imgs);

    let id = sqlx::query!(
        r#"
        INSERT INTO architectures
        (category, name, dynasty, location, description, cover_img, carousel_imgs)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
        payload.category,
        payload.name,
        payload.dynasty,
        payload.location,
        payload.description,
        payload.cover_img,
        carousel_imgs_json
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create architecture: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?
    .id;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

/// Deletes an architecture entry by ID.
/// Admin only.
pub async fn delete_architecture(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query!("DELETE FROM architectures WHERE id = ?", id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete architecture: {:?}", e);
            AppError::InternalServerError(e.to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Architecture not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
