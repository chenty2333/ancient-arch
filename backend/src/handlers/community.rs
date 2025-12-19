use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        post::{CreatePostRequest, Post, PostListParams},
        user::User,
    },
    utils::jwt::Claims,
};

/// Create a new post.
/// Requires: Login + (Verification OR Admin Role).
pub async fn create_post(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreatePostRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate payload
    if let Err(validation_errors) = payload.validate() {
        return Err(AppError::BadRequest(validation_errors.to_string()));
    }

    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    // 2. Check User Verification Status (DB Query)
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT 
            id, username, password, role, is_verified, 
            created_at
        FROM users 
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?
    .ok_or(AppError::NotFound("User not found".to_string()))?;

    // Rule: Must be verified OR admin to post
    if !user.is_verified && user.role != "admin" {
        return Err(AppError::AuthError(
            "You must be a verified contributor to post.".to_string(),
        ));
    }

    // 3. Insert Post
    let post_id = sqlx::query!(
        r#"
        INSERT INTO posts (user_id, title, content)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        user_id,
        payload.title,
        payload.content
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create post: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?
    .id;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({"id": post_id})),
    ))
}

/// List posts (Recent first).
/// Filter out soft-deleted posts.
/// Supports cursor-based pagination.
pub async fn list_posts(
    State(pool): State<PgPool>,
    Query(params): Query<PostListParams>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.unwrap_or(20).min(100); // Default 20, max 100

    // Since our database column created_at is TIMESTAMPTZ, sqlx maps it to DateTime<Utc> or NaiveDateTime depending on config.
    // Our Post struct uses NaiveDateTime.
    // But for the query parameter (cursor), we use DateTime<Utc> to handle the input cleanly.
    // We need to pass the cursor as matching the DB type.

    // NOTE: Because we reverted Post struct to use NaiveDateTime (due to issues),
    // but the DB column is now TIMESTAMPTZ (after migration),
    // SQLx might expect comparison with equivalent types.
    // TIMESTAMPTZ <-> DateTime<Utc> is standard.
    // Let's see if SQLx handles Option<DateTime<Utc>> against TIMESTAMPTZ correctly in the macro.

    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT 
            id, user_id, title, content, 
            created_at,
            updated_at,
            deleted_at,
            likes_count, comments_count, favorites_count
        FROM posts
        WHERE deleted_at IS NULL
          AND ($1::TIMESTAMPTZ IS NULL OR created_at < $1)
        ORDER BY created_at DESC
        LIMIT $2
        "#,
        params.cursor,
        limit
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list posts: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    Ok(Json(posts))
}

/// Get a single post by ID.
pub async fn get_post(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let post = sqlx::query_as!(
        Post,
        r#"
        SELECT 
            id, user_id, title, content, 
            created_at,
            updated_at,
            deleted_at,
            likes_count, comments_count, favorites_count
        FROM posts
        WHERE id = $1 AND deleted_at IS NULL
        "#,
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?
    .ok_or(AppError::NotFound("Post not found".to_string()))?;

    Ok(Json(post))
}

/// Delete a post (Soft Delete).
/// Requires: Login + (Author OR Admin).
pub async fn delete_post(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    // 1. Fetch Post to check ownership
    let post = sqlx::query!(
        "SELECT user_id FROM posts WHERE id = $1 AND deleted_at IS NULL",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?
    .ok_or(AppError::NotFound("Post not found".to_string()))?;

    // 2. Check Permission
    if post.user_id != user_id && claims.role != "admin" {
        return Err(AppError::AuthError(
            "You are not authorized to delete this post".to_string(),
        ));
    }

    // 3. Soft Delete
    sqlx::query!("UPDATE posts SET deleted_at = NOW() WHERE id = $1", id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete post: {:?}", e);
            AppError::InternalServerError(e.to_string())
        })?;

    Ok(StatusCode::NO_CONTENT)
}
