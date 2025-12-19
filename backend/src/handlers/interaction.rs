use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    error::AppError,
    models::comment::{CommentResponse, CreateCommentRequest},
    utils::jwt::Claims,
};

/// Toggle Like on a post.
pub async fn toggle_like(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // 1. Check if already liked
    let existing = sqlx::query!(
        "SELECT 1 as one FROM post_likes WHERE user_id = $1 AND post_id = $2",
        user_id,
        post_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let is_liked = existing.is_some();

    if is_liked {
        // Unlike
        sqlx::query!(
            "DELETE FROM post_likes WHERE user_id = $1 AND post_id = $2",
            user_id,
            post_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        sqlx::query!(
            "UPDATE posts SET likes_count = GREATEST(0, likes_count - 1) WHERE id = $1",
            post_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    } else {
        // Like
        sqlx::query!(
            "INSERT INTO post_likes (user_id, post_id) VALUES ($1, $2)",
            user_id,
            post_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique constraint") {
                // Concurrent request handled gracefully
                return AppError::Conflict("Already liked".to_string());
            }
            AppError::InternalServerError(e.to_string())
        })?;

        sqlx::query!(
            "UPDATE posts SET likes_count = likes_count + 1 WHERE id = $1",
            post_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(serde_json::json!({ "liked": !is_liked })))
}

/// Toggle Favorite on a post.
pub async fn toggle_favorite(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let existing = sqlx::query!(
        "SELECT 1 as one FROM post_favorites WHERE user_id = $1 AND post_id = $2",
        user_id,
        post_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let is_fav = existing.is_some();

    if is_fav {
        sqlx::query!(
            "DELETE FROM post_favorites WHERE user_id = $1 AND post_id = $2",
            user_id,
            post_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE posts SET favorites_count = GREATEST(0, favorites_count - 1) WHERE id = $1",
            post_id
        )
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query!(
            "INSERT INTO post_favorites (user_id, post_id) VALUES ($1, $2)",
            user_id,
            post_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE posts SET favorites_count = favorites_count + 1 WHERE id = $1",
            post_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(serde_json::json!({ "favorited": !is_fav })))
}

/// Create a new comment.
pub async fn create_comment(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<i64>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    let mut tx = pool.begin().await?;

    // 1. Logic for root_id and parent_id
    let mut root_id: Option<i64> = None;
    if let Some(pid) = payload.parent_id {
        // Fetch parent to find its root
        let parent = sqlx::query!(
            "SELECT id, root_id FROM comments WHERE id = $1 AND post_id = $2",
            pid,
            post_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("Parent comment not found".to_string()))?;

        // If parent has a root_id, then this new comment's root is that same root.
        // If parent's root_id is NULL, then the parent IS the root.
        root_id = Some(parent.root_id.unwrap_or(parent.id));
    }

    // 2. Insert Comment
    let new_id = sqlx::query!(
        r#"
        INSERT INTO comments (post_id, user_id, content, root_id, parent_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        post_id,
        user_id,
        payload.content,
        root_id,
        payload.parent_id
    )
    .fetch_one(&mut *tx)
    .await?
    .id;

    // 3. Update Post Count
    sqlx::query!(
        "UPDATE posts SET comments_count = comments_count + 1 WHERE id = $1",
        post_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": new_id })),
    ))
}

/// List all comments for a post.
pub async fn list_comments(
    State(pool): State<PgPool>,
    Path(post_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let comments = sqlx::query_as!(
        CommentResponse,
        r#"
        SELECT 
            c.id, c.post_id, c.user_id, u.username, c.content, 
            c.root_id, c.parent_id, c.created_at, c.deleted_at
        FROM comments c
        JOIN users u ON c.user_id = u.id
        WHERE c.post_id = $1 AND c.deleted_at IS NULL
        ORDER BY c.root_id IS NOT NULL, c.root_id, c.created_at ASC
        "#,
        post_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(comments))
}
