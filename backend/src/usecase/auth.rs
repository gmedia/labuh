use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::models::{AuthResponse, CreateUser, LoginRequest, User};
use crate::domain::user_repository::UserRepository;
use crate::error::{AppError, Result};
use crate::infrastructure::auth::jwt::JwtService;
use crate::infrastructure::auth::password::PasswordService;

pub struct AuthUsecase {
    repo: Arc<dyn UserRepository>,
    jwt_service: Arc<JwtService>,
}

impl AuthUsecase {
    pub fn new(repo: Arc<dyn UserRepository>, jwt_service: Arc<JwtService>) -> Self {
        Self { repo, jwt_service }
    }

    pub async fn register(&self, input: CreateUser) -> Result<AuthResponse> {
        // Only allow registration if no users exist (initial setup)
        let user_count = self.repo.count_users().await?;
        if user_count > 0 {
            return Err(AppError::Forbidden(
                "Public registration is disabled. Please contact your administrator.".to_string(),
            ));
        }

        // Check if user already exists (shouldn't happen if count == 0, but safety first)
        if self.repo.find_by_email(&input.email).await?.is_some() {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        // Hash password
        let password_hash = PasswordService::hash_password(&input.password)?;

        // Create user
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let user = User {
            id: id.clone(),
            email: input.email,
            password_hash,
            name: input.name,
            role: "admin".to_string(), // First user is always admin
            created_at: now.clone(),
            updated_at: now,
        };

        let created_user = self.repo.create(user).await?;
        let token = self.jwt_service.generate_token(
            &created_user.id,
            &created_user.email,
            &created_user.role,
        )?;

        Ok(AuthResponse {
            token,
            user: created_user.into(),
        })
    }

    pub async fn login(&self, input: LoginRequest) -> Result<AuthResponse> {
        let user = self
            .repo
            .find_by_email(&input.email)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        // Verify password
        PasswordService::verify_password(&input.password, &user.password_hash)?;

        let token = self
            .jwt_service
            .generate_token(&user.id, &user.email, &user.role)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<User> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound("User not found".to_string()))
    }

    pub fn verify_token(&self, token: &str) -> Result<crate::infrastructure::auth::jwt::Claims> {
        self.jwt_service.verify_token(token)
    }

    pub async fn is_setup_required(&self) -> Result<bool> {
        let count = self.repo.count_users().await?;
        Ok(count == 0)
    }
}
