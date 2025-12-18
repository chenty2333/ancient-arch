// src/utils/hash.rs

use crate::error::AppError;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

pub fn hash_password(password: &str) -> Result<String, AppError> {
    // Generate a random 128-bit salt.
    // Salt prevents rainbow table attacks by ensuring identical passwaors
    // result in different hashes.
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let result = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);

    match result {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
