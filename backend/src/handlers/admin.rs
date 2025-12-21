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
    utils::hash::hash_password,
    utils::jwt::Claims,
    utils::html::clean_html,
};

// --- DTOs ---

#[derive(Debug, Deserialize, Validate)]
pub struct AdminCreateUserRequest {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username length must be between 3 and 50 characters."
    ))]
    pub username: String,
    #[validate(length(
        min = 4,
        max = 128,
        message = "Password length must be between 4 and 128 characters."
    ))]
    pub password: String,
    pub role: String, // 'user' or 'admin'
}

#[derive(Debug, Deserialize, Validate)]
pub struct AdminUpdateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    #[validate(length(min = 1, max = 20))]
    pub role: Option<String>,
    #[validate(length(min = 4, max = 128))]
    pub password: Option<String>,
    pub is_verified: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewContributionRequest {
    pub status: String, // 'approved' or 'rejected'
    pub admin_comment: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateArchRequest {
    #[validate(length(min = 1, max = 50))]
    pub category: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub dynasty: Option<String>,
    #[validate(length(min = 1, max = 200))]
    pub location: Option<String>,
    #[validate(length(min = 1, max = 20000))]
    pub description: Option<String>,
    #[validate(length(min = 1, max = 500))]
    pub cover_img: Option<String>,
    #[validate(custom(function = validate_optional_carousel_urls))]
    pub carousel_imgs: Option<Vec<String>>,
}

fn validate_optional_carousel_urls(urls: &[String]) -> Result<(), validator::ValidationError> {
    for url in urls {
        if url.len() > 500 {
            return Err(validator::ValidationError::new("url_too_long"));
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateQuestionRequest {
    #[validate(length(min = 1, max = 20))]
    pub question_type: Option<String>,
    #[validate(length(min = 1, max = 1000))]
    pub content: Option<String>,
    #[validate(custom(function = validate_optional_options))]
    pub options: Option<Vec<String>>,
    #[validate(length(min = 1, max = 500))]
    pub answer: Option<String>,
    #[validate(length(max = 2000))]
    pub analysis: Option<String>,
}

fn validate_optional_options(options: &[String]) -> Result<(), validator::ValidationError> {
    for opt in options {
        if opt.len() > 500 {
            return Err(validator::ValidationError::new("option_too_long"));
        }
    }
    Ok(())
}

// --- User Management ---

pub async fn list_users(State(pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, '********' as "password!", role, is_verified, created_at
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

pub async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(payload): Json<AdminUpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE users SET ");
    let mut separated = builder.separated(", ");

    // If no fields provided, just return OK
    if payload.username.is_none()
        && payload.role.is_none()
        && payload.password.is_none()
        && payload.is_verified.is_none()
    {
        return Ok(StatusCode::OK);
    }

    if let Some(new_username) = payload.username {
        separated.push("username = ");
        separated.push_bind_unseparated(new_username);
    }
    if let Some(new_role) = payload.role {
        separated.push("role = ");
        separated.push_bind_unseparated(new_role);
    }
    if let Some(new_password) = payload.password {
        let hashed = hash_password(&new_password)?;
        separated.push("password = ");
        separated.push_bind_unseparated(hashed);
    }
    if let Some(verified) = payload.is_verified {
        separated.push("is_verified = ");
        separated.push_bind_unseparated(verified);
    }

    builder.push(" WHERE id = ");
    builder.push_bind(id);

    let result = builder.build().execute(&pool).await.map_err(|e| {
        if e.to_string().contains("unique constraint") {
            AppError::Conflict("Username already exists".to_string())
        } else {
            AppError::InternalServerError(e.to_string())
        }
    })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(StatusCode::OK)
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
        if e.to_string().contains("unique constraint") {
            AppError::Conflict("Username already exists".to_string())
        } else {
            AppError::InternalServerError(e.to_string())
        }
    })?
    .id;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
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

    let mut tx = pool.begin().await?;

    // 1. Fetch the ghost user ID for account deletion redirection
    let ghost_id = sqlx::query!("SELECT id FROM users WHERE username = 'ghost'")
        .fetch_optional(&mut *tx)
        .await?
        .map(|r| r.id)
        .ok_or_else(|| AppError::InternalServerError("Ghost user not found".to_string()))?;

    // Prevent deletion of the system-critical ghost user
    if id == ghost_id {
        return Err(AppError::BadRequest("Cannot delete the ghost user".to_string()));
    }

    // 2. Transfer posts to the ghost user
    sqlx::query!(
        "UPDATE posts SET user_id = $1 WHERE user_id = $2",
        ghost_id,
        id
    )
    .execute(&mut *tx)
    .await?;

    // 3. Transfer comments to the ghost user
    sqlx::query!(
        "UPDATE comments SET user_id = $1 WHERE user_id = $2",
        ghost_id,
        id
    )
    .execute(&mut *tx)
    .await?;
    
    // 4. Note on interactions (likes/favorites):
    // These are usually handled via ON DELETE CASCADE in the database schema.
    // We keep this behavior as likes are personal and don't need transfer.

    // 5. Delete the target user
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

// --- Architecture Management ---

pub async fn create_architecture(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateArchRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let carousel_json = serde_json::to_value(payload.carousel_imgs).unwrap_or_default();
    
    let clean_desc = clean_html(&payload.description);

    let id = sqlx::query!(
        r#"
        INSERT INTO architectures (category, name, dynasty, location, description, cover_img, carousel_imgs)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        payload.category, payload.name, payload.dynasty, payload.location, clean_desc, payload.cover_img, carousel_json
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
    payload.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

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
        separated.push_bind_unseparated(clean_html(&v));
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
    payload.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    let options_json = serde_json::to_value(payload.options).unwrap_or_default();
    
    let clean_content = clean_html(&payload.content);
    let clean_answer = clean_html(&payload.answer);
    let clean_analysis = payload.analysis.as_ref().map(|a| clean_html(a));

    let id = sqlx::query!(
        "INSERT INTO questions (type, content, options, answer, analysis) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        payload.question_type, clean_content, options_json, clean_answer, clean_analysis
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
    payload.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE questions SET ");
    let mut separated = builder.separated(", ");

    if let Some(v) = payload.question_type {
        separated.push("type = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = payload.content {
        separated.push("content = ");
        separated.push_bind_unseparated(clean_html(&v));
    }
    if let Some(v) = payload.options {
        separated.push("options = ");
        separated.push_bind_unseparated(serde_json::to_value(v).unwrap_or_default());
    }
    if let Some(v) = payload.answer {
        separated.push("answer = ");
        separated.push_bind_unseparated(clean_html(&v));
    }
    if let Some(v) = payload.analysis {
        separated.push("analysis = ");
        separated.push_bind_unseparated(clean_html(&v));
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
                let clean_desc = clean_html(&data.description);
                sqlx::query!(
                    "INSERT INTO architectures (category, name, dynasty, location, description, cover_img, carousel_imgs) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                    data.category, data.name, data.dynasty, data.location, clean_desc, data.cover_img, carousel
                ).execute(&mut *tx).await?;
            }
            "question" => {
                let data: CreateQuestionRequest = serde_json::from_value(contrib.data)?;
                let options = serde_json::to_value(data.options).unwrap_or_default();
                let clean_content = clean_html(&data.content);
                let clean_answer = clean_html(&data.answer);
                let clean_analysis = data.analysis.map(|a| clean_html(&a));
                
                sqlx::query!(
                    "INSERT INTO questions (type, content, options, answer, analysis) VALUES ($1, $2, $3, $4, $5)",
                    data.question_type, clean_content, options, clean_answer, clean_analysis
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