// src/models/architecture.rs

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Architecture {
    pub id: i64,
    pub category: String,
    pub name: String,
    pub dynasty: String,
    pub location: String,
    pub description: String,
    pub cover_img: String,
    pub carousel_imgs: Json<Vec<String>>,
}