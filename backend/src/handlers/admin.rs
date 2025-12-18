// src/handlers/admin.rs

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::{QueryBuilder, Postgres, PgPool};
use validator::Validate;

use crate::{
    error::AppError,
    models::{question::CreateQuestionRequest, user::User},
    utils::{hash::hash_password, jwt::Claims},
};

/// Lists all users in the system.
/// Admin only.
pub async fn list_users(State(pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT
        id, username, password, role,
        created_at::TEXT as "created_at: String"
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
#[derive(Debug, Deserialize, Validate)]
pub struct AdminCreateUserRequest {
    #[validate(length(min = 3, max = 20, message = "Username length must be between 3 and 20 characters."))]
    pub username: String,
    #[validate(length(min = 4, max = 20, message = "Password length must be between 4 and 20 characters."))]
    pub password: String,
    pub role: String, // 'user' or 'admin'
}

/// Creates a new user with specific role.
/// Admin only.
pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<AdminCreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(validation_errors) = payload.validate() {
        return Err(AppError::BadRequest(validation_errors.to_string()));
    }

    let hashed_password = hash_password(&payload.password)?;

    let id = sqlx::query!(
        r#"
        INSERT INTO users (username, password, role)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        payload.username,
        hashed_password,
        payload.role
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("unique constraint") || e.to_string().contains("23505") {
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
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(payload): Json<AdminUpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check existence
    let _exists = sqlx::query!("SELECT id FROM users WHERE id = $1", id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    // Perform updates sequentially if fields are present
    if let Some(new_username) = payload.username {
        sqlx::query!("UPDATE users SET username = $1 WHERE id = $2", new_username, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(new_role) = payload.role {
        sqlx::query!("UPDATE users SET role = $1 WHERE id = $2", new_role, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    if let Some(new_password) = payload.password {
        let hashed = hash_password(&new_password)?;
        sqlx::query!("UPDATE users SET password = $1 WHERE id = $2", hashed, id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    Ok(StatusCode::OK)
}

/// Deletes a user by ID.
/// Admin only. Prevents deleting self.
pub async fn delete_user(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    // Prevent self-deletion
    let current_user_id = claims.sub.parse::<i64>().unwrap_or(0);
    if id == current_user_id {
        return Err(AppError::BadRequest("Cannot delete yourself".to_string()));
    }

    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
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
    State(pool): State<PgPool>,
    Json(payload): Json<CreateArchRequest>,
) -> Result<impl IntoResponse, AppError> {
    let carousel_imgs_json = serde_json::to_value(payload.carousel_imgs).unwrap_or_default();

    let id = sqlx::query!(
        r#"
        INSERT INTO architectures
        (category, name, dynasty, location, description, cover_img, carousel_imgs)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
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
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateArchRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.category.is_none()
        && payload.name.is_none()
        && payload.dynasty.is_none()
        && payload.location.is_none()
        && payload.description.is_none()
        && payload.cover_img.is_none()
        && payload.carousel_imgs.is_none()
    {
        return Ok(StatusCode::OK);
    }

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE architectures SET ");
    let mut separated = builder.separated(", ");

    if let Some(category) = payload.category {
        separated.push("category = ");
        separated.push_bind_unseparated(category);
    }

    if let Some(name) = payload.name {
        separated.push("name = ");
        separated.push_bind_unseparated(name);
    }

    if let Some(dynasty) = payload.dynasty {
        separated.push("dynasty = ");
        separated.push_bind_unseparated(dynasty);
    }

    if let Some(location) = payload.location {
        separated.push("location = ");
        separated.push_bind_unseparated(location);
    }

    if let Some(description) = payload.description {
        separated.push("description = ");
        separated.push_bind_unseparated(description);
    }

    if let Some(cover_img) = payload.cover_img {
        separated.push("cover_img = ");
        separated.push_bind_unseparated(cover_img);
    }

    if let Some(carousel_imgs) = payload.carousel_imgs {
        separated.push("carousel_imgs = ");
        separated.push_bind_unseparated(serde_json::to_value(carousel_imgs).unwrap_or_default());
    }

    builder.push(" WHERE id = ");
    builder.push_bind(id);

    let result = builder.build().execute(&pool).await.map_err(|e| {
        tracing::error!("Failed to update architecture: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Architecture not found".to_string()));
    }

    Ok(StatusCode::OK)
}

/// Deletes an architecture entry by ID.
/// Admin only.
pub async fn delete_architecture(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query!("DELETE FROM architectures WHERE id = $1", id)
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
    State(pool): State<PgPool>,
    Json(payload): Json<CreateQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Serialize options as JSON
    let options_json = serde_json::to_value(payload.options).unwrap_or_default();

    let id = sqlx::query!(
        r#"
        INSERT INTO questions
        (type, content, options, answer, analysis)
        VALUES ($1, $2, $3, $4, $5)
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
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.question_type.is_none()
        && payload.content.is_none()
        && payload.options.is_none()
        && payload.answer.is_none()
        && payload.analysis.is_none()
    {
        return Ok(StatusCode::OK);
    }

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE questions SET ");
    let mut separated = builder.separated(", ");

    if let Some(q_type) = payload.question_type {
        separated.push("type = ");
        separated.push_bind_unseparated(q_type);
    }

    if let Some(content) = payload.content {
        separated.push("content = ");
        separated.push_bind_unseparated(content);
    }

    if let Some(options) = payload.options {
        separated.push("options = ");
        separated.push_bind_unseparated(serde_json::to_value(options).unwrap_or_default());
    }

    if let Some(answer) = payload.answer {
        separated.push("answer = ");
        separated.push_bind_unseparated(answer);
    }

    if let Some(analysis) = payload.analysis {
        separated.push("analysis = ");
        separated.push_bind_unseparated(analysis);
    }

    builder.push(" WHERE id = ");
    builder.push_bind(id);

    let result = builder.build().execute(&pool).await.map_err(|e| {
        tracing::error!("Failed to update question: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Question not found".to_string()));
    }

    Ok(StatusCode::OK)
}

/// Deletes a quiz question by ID.
/// Admin only.
pub async fn delete_question(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query!("DELETE FROM questions WHERE id = $1", id)
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