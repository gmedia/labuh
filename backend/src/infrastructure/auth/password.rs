use crate::error::{AppError, Result};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use password_hash::rand_core::OsRng;

pub struct PasswordService;

impl PasswordService {
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| AppError::Hash)
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<()> {
        let parsed_hash = PasswordHash::new(hash).map_err(|_| AppError::Hash)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::InvalidCredentials)
    }
}
