// src/models/architecture.rs

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use validator::Validate;
use url::Url;

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

/// DTO for creating a new architecture entry.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateArchRequest {
    #[validate(length(min = 1, max = 50))]
    pub category: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 50))]
    pub dynasty: String,
    #[validate(length(min = 1, max = 200))]
    pub location: String,
    #[validate(length(min = 1, max = 20000))]
    pub description: String,
    #[validate(length(min = 1, max = 500), custom(function = validate_url_string))]
    pub cover_img: String,
    #[validate(custom(function = validate_carousel_urls))]
    pub carousel_imgs: Vec<String>,
}

/// Validates that a string is a correctly formatted URL.
fn validate_url_string(url: &str) -> Result<(), validator::ValidationError> {
    if Url::parse(url).is_err() {
        return Err(validator::ValidationError::new("invalid_url"));
    }
    Ok(())
}

/// Validates a collection of URLs, ensuring each meets length and format requirements.
fn validate_carousel_urls(urls: &[String]) -> Result<(), validator::ValidationError> {
    for url in urls {
        if url.len() > 500 {
            return Err(validator::ValidationError::new("url_too_long"));
        }
        if Url::parse(url).is_err() {
             return Err(validator::ValidationError::new("invalid_url"));
        }
    }
    Ok(())
}
