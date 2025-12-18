// src/handlers/admin.rs

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::{question::CreateQuestionRequest, user::User},
    utils::{hash::hash_password, jwt::Claims},
};

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

/// DTO for Admin creating a user (can specify role).
#[derive(Debug, Deserialize)]
pub struct AdminCreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: String, // 'user' or 'admin'
}

/// Creates a new user with specific role.
/// Admin only.
pub async fn create_user(
    State(pool): State<SqlitePool>,
    Json(payload): Json<AdminCreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let hashed_password = hash_password(&payload.password)?;

    let id = sqlx::query!(
        r#"
        INSERT INTO users (username, password, role)
        VALUES (?, ?, ?)
        RETURNING id
        "#,
        payload.username,
        hashed_password,
        payload.role
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            AppError::Conflict(format!("Username '{}' already exists", payload.username))
        } else {
            tracing::error!("Failed to create user: {:?}", e);
            AppError::InternalServerError(e.to_string())
        }
    })?
    .id;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

/// DTO for updating a user. Fields are optional.
#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub username: Option<String>,
    pub role: Option<String>,
    pub password: Option<String>,
}

/// Updates user information.
/// Admin only.
pub async fn update_user(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<AdminUpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check existence
    let _exists = sqlx::query!("SELECT id FROM users WHERE id = ?", id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    // Perform updates sequentially if fields are present
    if let Some(new_username) = payload.username {
        sqlx::query!("UPDATE users SET username = ? WHERE id = ?", new_username, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(new_role) = payload.role {
        sqlx::query!("UPDATE users SET role = ? WHERE id = ?", new_role, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(new_password) = payload.password {
        let hashed = hash_password(&new_password)?;
        sqlx::query!("UPDATE users SET password = ? WHERE id = ?", hashed, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    Ok(StatusCode::OK)
}

/// Deletes a user by ID.
/// Admin only. Prevents deleting self.
pub async fn delete_user(
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    // Prevent self-deletion
    let current_user_id = claims.sub.parse::<i64>().unwrap_or(0);
    if id == current_user_id {
        return Err(AppError::BadRequest("Cannot delete yourself".to_string()));
    }

    let result = sqlx::query!("DELETE FROM users WHERE id = ?", id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete user: {:?}", e);
            AppError::InternalServerError(e.to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

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

/// DTO for updating an architecture entry. Fields are optional.
#[derive(Debug, Deserialize)]
pub struct UpdateArchRequest {
    pub category: Option<String>,
    pub name: Option<String>,
    pub dynasty: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub cover_img: Option<String>,
    pub carousel_imgs: Option<Vec<String>>,
}

/// Updates an architecture entry by ID.
/// Admin only.
pub async fn update_architecture(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateArchRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check existence
    let _exists = sqlx::query!("SELECT id FROM architectures WHERE id = ?", id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or(AppError::NotFound("Architecture not found".to_string()))?;

    if let Some(category) = payload.category {
        sqlx::query!("UPDATE architectures SET category = ? WHERE id = ?", category, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(name) = payload.name {
        sqlx::query!("UPDATE architectures SET name = ? WHERE id = ?", name, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(dynasty) = payload.dynasty {
        sqlx::query!("UPDATE architectures SET dynasty = ? WHERE id = ?", dynasty, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(location) = payload.location {
        sqlx::query!("UPDATE architectures SET location = ? WHERE id = ?", location, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(description) = payload.description {
        sqlx::query!("UPDATE architectures SET description = ? WHERE id = ?", description, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(cover_img) = payload.cover_img {
        sqlx::query!("UPDATE architectures SET cover_img = ? WHERE id = ?", cover_img, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(carousel_imgs) = payload.carousel_imgs {
        let imgs_json = sqlx::types::Json(carousel_imgs);
        sqlx::query!("UPDATE architectures SET carousel_imgs = ? WHERE id = ?", imgs_json, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    Ok(StatusCode::OK)
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

/// Creates a new quiz question.
/// Admin only.
pub async fn create_question(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Serialize options as JSON
    let options_json = sqlx::types::Json(payload.options);

    let id = sqlx::query!(
        r#"
        INSERT INTO questions
        (type, content, options, answer, analysis)
        VALUES (?, ?, ?, ?, ?)
        RETURNING id
        "#,
        payload.question_type,
        payload.content,
        options_json,
        payload.answer,
        payload.analysis
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create question: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?
    .id;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

/// DTO for updating a question. Fields are optional.
#[derive(Debug, Deserialize)]
pub struct UpdateQuestionRequest {
    pub question_type: Option<String>,
    pub content: Option<String>,
    pub options: Option<Vec<String>>,
    pub answer: Option<String>,
    pub analysis: Option<String>,
}

/// Updates a question by ID.
/// Admin only.
pub async fn update_question(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check existence
    let _exists = sqlx::query!("SELECT id FROM questions WHERE id = ?", id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or(AppError::NotFound("Question not found".to_string()))?;

    if let Some(q_type) = payload.question_type {
        sqlx::query!("UPDATE questions SET type = ? WHERE id = ?", q_type, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(content) = payload.content {
        sqlx::query!("UPDATE questions SET content = ? WHERE id = ?", content, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(options) = payload.options {
        let options_json = sqlx::types::Json(options);
        sqlx::query!("UPDATE questions SET options = ? WHERE id = ?", options_json, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(answer) = payload.answer {
        sqlx::query!("UPDATE questions SET answer = ? WHERE id = ?", answer, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(analysis) = payload.analysis {
        sqlx::query!("UPDATE questions SET analysis = ? WHERE id = ?", analysis, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    Ok(StatusCode::OK)
}

/// Deletes a quiz question by ID.
/// Admin only.
pub async fn delete_question(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query!("DELETE FROM questions WHERE id = ?", id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete question: {:?}", e);
            AppError::InternalServerError(e.to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Question not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
