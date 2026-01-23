//! Registry service for managing private registry credentials

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::RegistryCredential;

pub struct RegistryService {
    db: SqlitePool,
}

impl RegistryService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Simple encoding for passwords (NOT secure encryption - use a proper vault in production)
    fn encode_password(password: &str) -> String {
        BASE64.encode(password.as_bytes())
    }

    /// Decode password
    fn decode_password(encoded: &str) -> Result<String> {
        let bytes = BASE64.decode(encoded)
            .map_err(|_| AppError::Internal("Failed to decode password".to_string()))?;
        String::from_utf8(bytes)
            .map_err(|_| AppError::Internal("Invalid password encoding".to_string()))
    }

    /// List all credentials for a user (passwords hidden)
    pub async fn list_credentials(&self, user_id: &str) -> Result<Vec<RegistryCredential>> {
        let credentials = sqlx::query_as::<_, RegistryCredential>(
            "SELECT * FROM registry_credentials WHERE user_id = ? ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(credentials)
    }

    /// Add a new credential
    pub async fn add_credential(
        &self,
        user_id: &str,
        name: &str,
        registry_url: &str,
        username: &str,
        password: &str,
    ) -> Result<RegistryCredential> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let password_encrypted = Self::encode_password(password);

        sqlx::query(
            "INSERT INTO registry_credentials (id, user_id, name, registry_url, username, password_encrypted, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(user_id)
        .bind(name)
        .bind(registry_url)
        .bind(username)
        .bind(&password_encrypted)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        self.get_credential(&id, user_id).await
    }

    /// Get a single credential
    pub async fn get_credential(&self, id: &str, user_id: &str) -> Result<RegistryCredential> {
        let credential = sqlx::query_as::<_, RegistryCredential>(
            "SELECT * FROM registry_credentials WHERE id = ? AND user_id = ?"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Credential not found".to_string()))?;

        Ok(credential)
    }

    /// Remove a credential
    pub async fn remove_credential(&self, id: &str, user_id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM registry_credentials WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Credential not found".to_string()));
        }

        Ok(())
    }

    /// Get credentials for a specific registry (used during image pull)
    pub async fn get_credentials_for_registry(&self, user_id: &str, image: &str) -> Result<Option<(String, String)>> {
        // Extract registry from image name
        let registry_url = Self::extract_registry_from_image(image);

        let credential = sqlx::query_as::<_, RegistryCredential>(
            "SELECT * FROM registry_credentials WHERE user_id = ? AND registry_url = ? LIMIT 1"
        )
        .bind(user_id)
        .bind(&registry_url)
        .fetch_optional(&self.db)
        .await?;

        if let Some(cred) = credential {
            let password = Self::decode_password(&cred.password_encrypted)?;
            return Ok(Some((cred.username, password)));
        }

        Ok(None)
    }

    /// Extract registry URL from image name
    fn extract_registry_from_image(image: &str) -> String {
        // Examples:
        // nginx -> docker.io
        // username/repo -> docker.io
        // ghcr.io/owner/repo -> ghcr.io
        // registry.example.com/repo -> registry.example.com

        if !image.contains('/') {
            // Official Docker Hub image like "nginx"
            return "docker.io".to_string();
        }

        let parts: Vec<&str> = image.split('/').collect();
        if parts.len() >= 2 {
            let first = parts[0];
            // Check if first part looks like a registry (contains . or :)
            if first.contains('.') || first.contains(':') {
                return first.to_string();
            }
        }

        // Default to Docker Hub
        "docker.io".to_string()
    }
}
