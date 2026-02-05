use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::models::{RegistryCredential, RegistryCredentialResponse, TeamRole};
use crate::domain::registry_repository::RegistryRepository;
use crate::error::{AppError, Result};

pub struct RegistryUsecase {
    repo: Arc<dyn RegistryRepository>,
    team_repo: Arc<dyn crate::domain::TeamRepository>,
}

impl RegistryUsecase {
    pub fn new(
        repo: Arc<dyn RegistryRepository>,
        team_repo: Arc<dyn crate::domain::TeamRepository>,
    ) -> Self {
        Self { repo, team_repo }
    }

    /// Verify user has required role for the team
    async fn verify_permission(
        &self,
        team_id: &str,
        user_id: &str,
        required_role: TeamRole,
    ) -> Result<()> {
        let role = self
            .team_repo
            .get_user_role(team_id, user_id)
            .await?
            .ok_or(AppError::Forbidden("Access denied".to_string()))?;

        let role_priority = |r: TeamRole| match r {
            TeamRole::Owner => 4,
            TeamRole::Admin => 3,
            TeamRole::Developer => 2,
            TeamRole::Viewer => 1,
        };

        if role_priority(role) < role_priority(required_role) {
            return Err(AppError::Forbidden(
                "Insufficient permissions for this operation".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn list_credentials(
        &self,
        team_id: &str,
        user_id: &str,
    ) -> Result<Vec<RegistryCredentialResponse>> {
        self.verify_permission(team_id, user_id, TeamRole::Viewer)
            .await?;

        let creds = self.repo.list_by_team(team_id).await?;
        Ok(creds.into_iter().map(Into::into).collect())
    }

    pub async fn add_credential(
        &self,
        user_id: &str,
        team_id: &str,
        name: &str,
        registry_url: &str,
        username: &str,
        password: &str,
    ) -> Result<RegistryCredentialResponse> {
        self.verify_permission(team_id, user_id, TeamRole::Admin)
            .await?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let password_encrypted = BASE64.encode(password.as_bytes());

        let cred = RegistryCredential {
            id,
            user_id: user_id.to_string(),
            team_id: team_id.to_string(),
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

    pub async fn remove_credential(&self, id: &str, team_id: &str, user_id: &str) -> Result<()> {
        self.verify_permission(team_id, user_id, TeamRole::Admin)
            .await?;

        self.repo.delete(id, team_id).await
    }

    pub async fn get_credentials_for_image(
        &self,
        _user_id: &str,
        team_id: &str,
        image: &str,
    ) -> Result<Option<(String, String)>> {
        self.get_credentials_for_image_internal(team_id, image)
            .await
    }

    pub async fn get_credentials_for_image_internal(
        &self,
        team_id: &str,
        image: &str,
    ) -> Result<Option<(String, String)>> {
        let url = self.extract_registry(image);
        let cred = self.repo.find_by_url(team_id, &url).await?;

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
