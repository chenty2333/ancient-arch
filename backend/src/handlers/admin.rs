// src/handlers/admin.rs

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::{PgPool, Postgres, QueryBuilder};
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        architecture::CreateArchRequest, contribution::Contribution,
        question::CreateQuestionRequest, user::User,
    },
    utils::{hash::hash_password, jwt::Claims},
};

// --- DTOs ---

#[derive(Debug, Deserialize, Validate)]
pub struct AdminCreateUserRequest {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Username length must be between 3 and 20 characters."
    ))]
    pub username: String,
    #[validate(length(
        min = 4,
        max = 20,
        message = "Password length must be between 4 and 20 characters."
    ))]
    pub password: String,
    pub role: String, // 'user' or 'admin'
}

#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub username: Option<String>,
    pub role: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewContributionRequest {
    pub status: String, // 'approved' or 'rejected'
    pub admin_comment: Option<String>,
}

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

#[derive(Debug, Deserialize)]
pub struct UpdateQuestionRequest {
    pub question_type: Option<String>,
    pub content: Option<String>,
    pub options: Option<Vec<String>>,
    pub answer: Option<String>,
    pub analysis: Option<String>,
}

// --- User Management ---

pub async fn list_users(State(pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, password, role, is_verified, created_at
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

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<AdminCreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let hashed_password = hash_password(&payload.password)?;

    let id = sqlx::query!(
        "INSERT INTO users (username, password, role) VALUES ($1, $2, $3) RETURNING id",
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
            AppError::InternalServerError(e.to_string())
        }
    })?
    .id;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

pub async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(payload): Json<AdminUpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Sequential updates for simplicity in Admin panel
    if let Some(new_username) = payload.username {
        sqlx::query!(
            "UPDATE users SET username = $1 WHERE id = $2",
            new_username,
            id
        )
        .execute(&pool)
        .await?;
    }
    if let Some(new_role) = payload.role {
        sqlx::query!("UPDATE users SET role = $1 WHERE id = $2", new_role, id)
            .execute(&pool)
            .await?;
    }
    if let Some(new_password) = payload.password {
        let hashed = hash_password(&new_password)?;
        sqlx::query!("UPDATE users SET password = $1 WHERE id = $2", hashed, id)
            .execute(&pool)
            .await?;
    }
    Ok(StatusCode::OK)
}

pub async fn delete_user(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let current_user_id = claims.sub.parse::<i64>().unwrap_or(0);
    if id == current_user_id {
        return Err(AppError::BadRequest("Cannot delete yourself".to_string()));
    }

    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

// --- Architecture Management ---

pub async fn create_architecture(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateArchRequest>,
) -> Result<impl IntoResponse, AppError> {
    let carousel_json = serde_json::to_value(payload.carousel_imgs).unwrap_or_default();
    let id = sqlx::query!(
        r#"
        INSERT INTO architectures (category, name, dynasty, location, description, cover_img, carousel_imgs)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        payload.category, payload.name, payload.dynasty, payload.location, payload.description, payload.cover_img, carousel_json
    )
    .fetch_one(&pool)
    .await?
    .id;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

pub async fn update_architecture(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateArchRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE architectures SET ");
    let mut separated = builder.separated(", ");

    if let Some(v) = payload.category {
        separated.push("category = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = payload.name {
        separated.push("name = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = payload.dynasty {
        separated.push("dynasty = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = payload.location {
        separated.push("location = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = payload.description {
        separated.push("description = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = payload.cover_img {
        separated.push("cover_img = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = payload.carousel_imgs {
        separated.push("carousel_imgs = ");
        separated.push_bind_unseparated(serde_json::to_value(v).unwrap_or_default());
    }

    builder.push(" WHERE id = ");
    builder.push_bind(id);

    let result = builder.build().execute(&pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Architecture not found".to_string()));
    }
    Ok(StatusCode::OK)
}

pub async fn delete_architecture(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query!("DELETE FROM architectures WHERE id = $1", id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Architecture not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

// --- Question Management ---

pub async fn create_question(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let options_json = serde_json::to_value(payload.options).unwrap_or_default();
    let id = sqlx::query!(
        "INSERT INTO questions (type, content, options, answer, analysis) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        payload.question_type, payload.content, options_json, payload.answer, payload.analysis
    )
    .fetch_one(&pool)
    .await?
    .id;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

pub async fn update_question(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE questions SET ");
    let mut separated = builder.separated(", ");

    if let Some(v) = payload.question_type {
        separated.push("type = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = payload.content {
        separated.push("content = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = payload.options {
        separated.push("options = ");
        separated.push_bind_unseparated(serde_json::to_value(v).unwrap_or_default());
    }
    if let Some(v) = payload.answer {
        separated.push("answer = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = payload.analysis {
        separated.push("analysis = ");
        separated.push_bind_unseparated(v);
    }

    builder.push(" WHERE id = ");
    builder.push_bind(id);

    let result = builder.build().execute(&pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Question not found".to_string()));
    }
    Ok(StatusCode::OK)
}

pub async fn delete_question(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query!("DELETE FROM questions WHERE id = $1", id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Question not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

// --- Contribution Management ---

/// Lists all contributions.
pub async fn list_contributions(State(pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    let list = sqlx::query_as!(
        Contribution,
        "SELECT id, user_id, type, data, status, admin_comment, created_at, reviewed_at FROM contributions ORDER BY created_at ASC"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(list))
}

/// Reviews a contribution (Approve/Reject).
pub async fn review_contribution(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(payload): Json<ReviewContributionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut tx = pool.begin().await?;

    let contrib = sqlx::query_as!(
        Contribution,
        "SELECT * FROM contributions WHERE id = $1 AND status = 'pending'",
        id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound(
        "Pending contribution not found".to_string(),
    ))?;

    if payload.status == "approved" {
        match contrib.r#type.as_str() {
            "architecture" => {
                let data: CreateArchRequest = serde_json::from_value(contrib.data)?;
                let carousel = serde_json::to_value(data.carousel_imgs).unwrap_or_default();
                sqlx::query!(
                    "INSERT INTO architectures (category, name, dynasty, location, description, cover_img, carousel_imgs) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                    data.category, data.name, data.dynasty, data.location, data.description, data.cover_img, carousel
                ).execute(&mut *tx).await?;
            }
            "question" => {
                let data: CreateQuestionRequest = serde_json::from_value(contrib.data)?;
                let options = serde_json::to_value(data.options).unwrap_or_default();
                sqlx::query!(
                    "INSERT INTO questions (type, content, options, answer, analysis) VALUES ($1, $2, $3, $4, $5)",
                    data.question_type, data.content, options, data.answer, data.analysis
                ).execute(&mut *tx).await?;
            }
            _ => return Err(AppError::BadRequest("Unknown type".to_string())),
        }
    }

    sqlx::query!(
        "UPDATE contributions SET status = $1, admin_comment = $2, reviewed_at = NOW() WHERE id = $3",
        payload.status, payload.admin_comment, id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(StatusCode::OK)
}
