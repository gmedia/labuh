#![allow(dead_code)]
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::models::RegistryCredential;
use crate::error::{AppError, Result};

#[derive(serde::Deserialize)]
pub struct CreateRegistryCredential {
    pub name: String,
    pub registry_url: String,
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct RegistryCredentialResponse {
    pub id: String,
    pub name: String,
    pub registry_url: String,
    pub username: String,
    pub created_at: String,
}

impl From<RegistryCredential> for RegistryCredentialResponse {
    fn from(creds: RegistryCredential) -> Self {
        Self {
            id: creds.id,
            name: creds.name,
            registry_url: creds.registry_url,
            username: creds.username,
            created_at: creds.created_at,
        }
    }
}

pub struct RegistryService {
    pool: SqlitePool,
}

impl RegistryService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn encode_password(password: &str) -> String {
        BASE64.encode(password)
    }

    pub fn decode_password(encoded: &str) -> Result<String> {
        let decoded = BASE64.decode(encoded).map_err(|_| AppError::Hash)?;
        String::from_utf8(decoded).map_err(|_| AppError::Hash)
    }

    pub async fn list_credentials(&self, user_id: &str) -> Result<Vec<RegistryCredential>> {
        let credentials = sqlx::query_as::<_, RegistryCredential>(
            "SELECT * FROM registry_credentials WHERE user_id = ? ORDER BY registry_url",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(credentials)
    }

    pub async fn add_credential(
        &self,
        user_id: &str,
        input: CreateRegistryCredential,
    ) -> Result<RegistryCredential> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let encoded_password = Self::encode_password(&input.password);

        sqlx::query(
            r#"
            INSERT INTO registry_credentials (id, user_id, name, registry_url, username, password_encrypted, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(user_id)
        .bind(&input.name)
        .bind(&input.registry_url)
        .bind(&input.username)
        .bind(&encoded_password)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        self.get_credential(&id, user_id).await
    }

    pub async fn get_credential(&self, id: &str, user_id: &str) -> Result<RegistryCredential> {
        sqlx::query_as::<_, RegistryCredential>(
            "SELECT * FROM registry_credentials WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Credential not found".to_string()))
    }

    pub async fn get_credential_by_url(
        &self,
        user_id: &str,
        url: &str,
    ) -> Result<Option<RegistryCredential>> {
        let cred = sqlx::query_as::<_, RegistryCredential>(
            "SELECT * FROM registry_credentials WHERE user_id = ? AND registry_url = ?",
        )
        .bind(user_id)
        .bind(url)
        .fetch_optional(&self.pool)
        .await?;

        Ok(cred)
    }

    pub async fn remove_credential(&self, id: &str, user_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM registry_credentials WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
