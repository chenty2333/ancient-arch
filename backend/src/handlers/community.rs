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
    models::post::{CreatePostRequest, Post, PostListParams},
    utils::jwt::{Claims, VerifiedUser},
};

/// Create a new post.
/// Automatically restricted to Verified users or Admins via the VerifiedUser extractor.
pub async fn create_post(
    State(pool): State<PgPool>,
    user: VerifiedUser,
    Json(payload): Json<CreatePostRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate payload
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // 2. Insert Post (Permissions already checked by VerifiedUser extractor)
    let post_id = sqlx::query!(
        r#"
        INSERT INTO posts (user_id, title, content)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        user.id,
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
    let limit = params.limit.unwrap_or(20).min(100);
    let sort = params.sort.unwrap_or_else(|| "new".to_string());

    let posts = if sort == "hot" {
        sqlx::query_as!(
            Post,
            r#"
            SELECT 
                id, user_id, title, content, 
                created_at, updated_at, deleted_at,
                likes_count, comments_count, favorites_count,
                FALSE as "is_liked!", FALSE as "is_favorited!"
            FROM posts
            WHERE deleted_at IS NULL
            ORDER BY (
                (likes_count * 5 + comments_count * 3 + favorites_count * 10)::FLOAT / 
                POW(EXTRACT(EPOCH FROM (NOW() - created_at)) / 3600 + 2, 1.5)
            ) DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list posts (hot): {:?}", e);
            AppError::InternalServerError(e.to_string())
        })?
    } else {
        sqlx::query_as!(
            Post,
            r#"
            SELECT 
                id, user_id, title, content, 
                created_at, updated_at, deleted_at,
                likes_count, comments_count, favorites_count,
                FALSE as "is_liked!", FALSE as "is_favorited!"
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
            tracing::error!("Failed to list posts (new): {:?}", e);
            AppError::InternalServerError(e.to_string())
        })?
    };

    Ok(Json(posts))
}

/// Get a single post by ID.
pub async fn get_post(
    State(pool): State<PgPool>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.map(|c| c.sub.parse::<i64>().unwrap_or(0));

    let post = if let Some(uid) = user_id {
        sqlx::query_as!(
            Post,
            r#"
            SELECT 
                p.id, p.user_id, p.title, p.content, 
                p.created_at, p.updated_at, p.deleted_at,
                p.likes_count, p.comments_count, p.favorites_count,
                (EXISTS (SELECT 1 FROM post_likes WHERE user_id = $2 AND post_id = p.id)) as "is_liked!",
                (EXISTS (SELECT 1 FROM post_favorites WHERE user_id = $2 AND post_id = p.id)) as "is_favorited!"
            FROM posts p
            WHERE p.id = $1 AND p.deleted_at IS NULL
            "#,
            id,
            uid
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch post details (auth): {:?}", e);
            AppError::InternalServerError(e.to_string())
        })?
    } else {
        sqlx::query_as!(
            Post,
            r#"
            SELECT 
                id, user_id, title, content, 
                created_at, updated_at, deleted_at,
                likes_count, comments_count, favorites_count,
                FALSE as "is_liked!", FALSE as "is_favorited!"
            FROM posts
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch post details: {:?}", e);
            AppError::InternalServerError(e.to_string())
        })?
    };

    let post = post.ok_or(AppError::NotFound("Post not found".to_string()))?;
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
