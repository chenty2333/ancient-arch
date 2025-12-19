use axum::{
    Extension, Json,
    extract::{Query, State},
    response::IntoResponse,
};
use sqlx::PgPool;

use crate::{
    error::AppError,
    models::{
        contribution::Contribution,
        post::{Post, PostListParams},
        user::{FavoritePostResponse, MeResponse},
    },
    utils::jwt::Claims,
};

/// Get current user's profile and statistics.
pub async fn get_me(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    // Using subqueries for counts is efficient given our indexes on user_id and post_id.
    let me = sqlx::query!(
        r#"
        SELECT 
            u.id, u.username, u.role, u.is_verified, u.created_at,
            (SELECT COUNT(*) FROM posts WHERE user_id = u.id AND deleted_at IS NULL) as posts_count,
            (SELECT COUNT(*) FROM post_likes pl JOIN posts p ON pl.post_id = p.id WHERE p.user_id = u.id) as total_likes_received
        FROM users u
        WHERE u.id = $1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(MeResponse {
        id: me.id,
        username: me.username,
        role: me.role,
        is_verified: me.is_verified,
        created_at: me.created_at,
        posts_count: me.posts_count.unwrap_or(0),
        total_likes_received: me.total_likes_received.unwrap_or(0),
    }))
}

/// List posts created by the current user.
/// Includes real interaction status (is_liked, is_favorited).
pub async fn list_my_posts(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PostListParams>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub.parse::<i64>().unwrap_or(0);
    let limit = params.limit.unwrap_or(20).min(100);

    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT 
            p.id, p.user_id, p.title, p.content, 
            p.created_at, p.updated_at, p.deleted_at,
            p.likes_count, p.comments_count, p.favorites_count,
            (pl.user_id IS NOT NULL) as "is_liked!",
            (pf.user_id IS NOT NULL) as "is_favorited!"
        FROM posts p
        LEFT JOIN post_likes pl ON p.id = pl.post_id AND pl.user_id = $1
        LEFT JOIN post_favorites pf ON p.id = pf.post_id AND pf.user_id = $1
        WHERE p.user_id = $1 AND p.deleted_at IS NULL
          AND ($2::TIMESTAMPTZ IS NULL OR p.created_at < $2)
        ORDER BY p.created_at DESC
        LIMIT $3
        "#,
        user_id,
        params.cursor,
        limit
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(posts))
}

/// List posts favorited by the current user.
pub async fn list_my_favorites(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    let favorites = sqlx::query_as!(
        FavoritePostResponse,
        r#"
        SELECT 
            f.post_id, p.title, u.username as author_username, 
            f.created_at as favorited_at
        FROM post_favorites f
        JOIN posts p ON f.post_id = p.id
        JOIN users u ON p.user_id = u.id
        WHERE f.user_id = $1 AND p.deleted_at IS NULL
        ORDER BY f.created_at DESC
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(favorites))
}

/// List contribution history of the current user.
/// This is open to all logged-in users to view their own history.
pub async fn list_my_contributions(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub.parse::<i64>().unwrap_or(0);

    let list = sqlx::query_as!(
        Contribution,
        r#"
        SELECT id, user_id, type, data, status, admin_comment, created_at, reviewed_at
        FROM contributions
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(list))
}
