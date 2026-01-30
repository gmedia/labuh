#![allow(dead_code)]
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::models::registry::{RegistryCredential, RegistryCredentialResponse};
use crate::domain::registry_repository::RegistryRepository;
use crate::error::{AppError, Result};

pub struct RegistryUsecase {
    repo: Arc<dyn RegistryRepository>,
}

impl RegistryUsecase {
    pub fn new(repo: Arc<dyn RegistryRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_credentials(&self, user_id: &str) -> Result<Vec<RegistryCredentialResponse>> {
        let creds = self.repo.list_by_user(user_id).await?;
        Ok(creds.into_iter().map(Into::into).collect())
    }

    pub async fn add_credential(
        &self,
        user_id: &str,
        name: &str,
        registry_url: &str,
        username: &str,
        password: &str,
    ) -> Result<RegistryCredentialResponse> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let password_encrypted = BASE64.encode(password.as_bytes());

        let cred = RegistryCredential {
            id,
            user_id: user_id.to_string(),
            name: name.to_string(),
            registry_url: registry_url.to_string(),
            username: username.to_string(),
            password_encrypted,
            created_at: now.clone(),
            updated_at: now,
        };

        let saved = self.repo.save(cred).await?;
        Ok(saved.into())
    }

    pub async fn remove_credential(&self, id: &str, user_id: &str) -> Result<()> {
        self.repo.delete(id, user_id).await
    }

    pub async fn get_credentials_for_image(
        &self,
        user_id: &str,
        image: &str,
    ) -> Result<Option<(String, String)>> {
        let url = self.extract_registry(image);
        let cred = self.repo.find_by_url(user_id, &url).await?;

        if let Some(c) = cred {
            let password = String::from_utf8(
                BASE64
                    .decode(&c.password_encrypted)
                    .map_err(|_| AppError::Internal("Failed to decode password".to_string()))?,
            )
            .map_err(|_| AppError::Internal("Invalid password encoding".to_string()))?;

            return Ok(Some((c.username, password)));
        }

        Ok(None)
    }

    fn extract_registry(&self, image: &str) -> String {
        if !image.contains('/') {
            return "docker.io".to_string();
        }

        let parts: Vec<&str> = image.split('/').collect();
        if parts.len() >= 2 {
            let first = parts[0];
            if first.contains('.') || first.contains(':') {
                return first.to_string();
            }
        }

        "docker.io".to_string()
    }
}
