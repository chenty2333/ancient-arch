// src/handlers/architecture.rs

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{error::AppError, models::architecture::Architecture};

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub category: Option<String>,
}

pub async fn list_architectures(
    State(pool): State<SqlitePool>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, AppError> {
    let architectures = if let Some(cat) = params.category {
        sqlx::query_as!(
            Architecture,
            r#"
            SELECT id, category, name, dynasty, location, description, cover_img, carousel_imgs as "carousel_imgs:sqlx::types::Json<Vec<String>>"
            FROM architectures
            WHERE category = ?
            "#,
            cat
        )
        .fetch_all(&pool)
        .await?
    } else {
        sqlx::query_as!(
            Architecture,
            r#"
            SELECT id, category, name, dynasty, location, description, cover_img, carousel_imgs as "carousel_imgs:sqlx::types::Json<Vec<String>>"
            FROM architectures
            "#
        )
        .fetch_all(&pool)
        .await?
    };

    Ok(Json(architectures))
}

pub async fn get_architecture(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let architecture = sqlx::query_as!(
        Architecture,
        r#"
        SELECT id, category, name, dynasty, location, description, cover_img, carousel_imgs as "carousel_imgs:sqlx::types::Json<Vec<String>>"
        FROM architectures
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound("Architecture not found".to_string()))?;

    Ok(Json(architecture))
}
