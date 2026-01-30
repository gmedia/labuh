use crate::domain::models::registry::RegistryCredential;
use crate::domain::registry_repository::RegistryRepository;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;

pub struct SqliteRegistryRepository {
    pool: SqlitePool,
}

impl SqliteRegistryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RegistryRepository for SqliteRegistryRepository {
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<RegistryCredential>> {
        let credentials = sqlx::query_as::<_, RegistryCredential>(
            "SELECT * FROM registry_credentials WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(credentials)
    }

    async fn find_by_id(&self, id: &str, user_id: &str) -> Result<RegistryCredential> {
        let credential = sqlx::query_as::<_, RegistryCredential>(
            "SELECT * FROM registry_credentials WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Credential not found".to_string()))?;

        Ok(credential)
    }

    async fn find_by_url(&self, user_id: &str, url: &str) -> Result<Option<RegistryCredential>> {
        let credential = sqlx::query_as::<_, RegistryCredential>(
            "SELECT * FROM registry_credentials WHERE user_id = ? AND registry_url = ? LIMIT 1",
        )
        .bind(user_id)
        .bind(url)
        .fetch_optional(&self.pool)
        .await?;

        Ok(credential)
    }

    async fn save(&self, cred: RegistryCredential) -> Result<RegistryCredential> {
        // Upsert or insert/update
        let existing = sqlx::query("SELECT id FROM registry_credentials WHERE id = ?")
            .bind(&cred.id)
            .fetch_optional(&self.pool)
            .await?;

        if existing.is_some() {
            sqlx::query(
                "UPDATE registry_credentials SET name = ?, registry_url = ?, username = ?, password_encrypted = ?, updated_at = ? WHERE id = ?"
            )
            .bind(&cred.name)
            .bind(&cred.registry_url)
            .bind(&cred.username)
            .bind(&cred.password_encrypted)
            .bind(&cred.updated_at)
            .bind(&cred.id)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                "INSERT INTO registry_credentials (id, user_id, name, registry_url, username, password_encrypted, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&cred.id)
            .bind(&cred.user_id)
            .bind(&cred.name)
            .bind(&cred.registry_url)
            .bind(&cred.username)
            .bind(&cred.password_encrypted)
            .bind(&cred.created_at)
            .bind(&cred.updated_at)
            .execute(&self.pool)
            .await?;
        }

        Ok(cred)
    }

    async fn delete(&self, id: &str, user_id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM registry_credentials WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Credential not found".to_string()));
        }

        Ok(())
    }
}
