use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use password_hash::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::{AuthResponse, CreateUser, LoginRequest, User};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct AuthService {
    pool: SqlitePool,
    jwt_secret: String,
    jwt_expiration_hours: u64,
}

impl AuthService {
    pub fn new(pool: SqlitePool, jwt_secret: String, jwt_expiration_hours: u64) -> Self {
        Self {
            pool,
            jwt_secret,
            jwt_expiration_hours,
        }
    }

    pub async fn register(&self, input: CreateUser) -> Result<AuthResponse> {
        // Check if user already exists
        let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(&input.email)
            .fetch_one(&self.pool)
            .await?;

        if existing > 0 {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        // Hash password
        let password_hash = self.hash_password(&input.password)?;

        // Create user
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, name, role, created_at, updated_at)
            VALUES (?, ?, ?, ?, 'user', ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&input.email)
        .bind(&password_hash)
        .bind(&input.name)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let user = self.get_user_by_id(&id).await?;
        let token = self.generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }

    pub async fn login(&self, input: LoginRequest) -> Result<AuthResponse> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(&input.email)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        // Verify password
        self.verify_password(&input.password, &user.password_hash)?;

        let token = self.generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::NotFound("User not found".to_string()))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    fn generate_token(&self, user: &User) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.jwt_expiration_hours as i64);

        let claims = Claims {
            sub: user.id.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(AppError::Jwt)
    }

    fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| AppError::Hash)
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<()> {
        let parsed_hash = PasswordHash::new(hash).map_err(|_| AppError::Hash)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::InvalidCredentials)
    }
}
