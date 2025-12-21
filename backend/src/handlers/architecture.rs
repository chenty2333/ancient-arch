// src/handlers/architecture.rs

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{error::AppError, models::architecture::Architecture};

/// Query parameters for listing architectures.
#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub category: Option<String>,
    pub q: Option<String>,
}

/// Lists all architectures, optionally filtered by category and search keyword.
pub async fn list_architectures(
    State(pool): State<PgPool>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, AppError> {
    // Prepare search pattern
    let search_pattern = params.q.map(|k| format!("%{}%", k));

    // Unified query handling optional filters
    let architectures = sqlx::query_as!(
        Architecture,
        r#"
        SELECT id, category, name, dynasty, location, description, cover_img, carousel_imgs as "carousel_imgs: sqlx::types::Json<Vec<String>>"
        FROM architectures
        WHERE ($1::TEXT IS NULL OR category = $1)
          AND ($2::TEXT IS NULL OR name ILIKE $2)
        "#,
        params.category,
        search_pattern
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(architectures))
}

/// Retrieves a single architecture by ID.
pub async fn get_architecture(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let architecture = sqlx::query_as!(
        Architecture,
        r#"
                    SELECT id, category, name, dynasty, location, description, cover_img, carousel_imgs as "carousel_imgs: sqlx::types::Json<Vec<String>>"
        FROM architectures
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound("Architecture not found".to_string()))?;

    Ok(Json(architecture))
}
