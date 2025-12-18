// src/models/architecture.rs

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};

/// Represents the 'architectures' table in the database.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Architecture {
    pub id: i64,

    /// Architecture category (e.g., "Palace", "Bridge").
    pub category: String,

    pub name: String,

    /// Historical dynasty (e.g., "Ming", "Qing").
    pub dynasty: String,

    pub location: String,

    pub description: String,

    /// URL to the cover image.
    pub cover_img: String,

    /// List of carousel image URLs.
    /// Stored as a JSON array in the database.
    /// `sqlx::types::Json` handles automatic serialization/deserialization.
    pub carousel_imgs: Json<Vec<String>>,
}
