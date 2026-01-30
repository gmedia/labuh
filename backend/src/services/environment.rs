#![allow(dead_code)]
//! Environment service for managing stack environment variables

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::models::StackEnvVar;
use crate::error::{AppError, Result};

#[derive(serde::Deserialize)]
pub struct SetEnvVarRequest {
    pub container_name: String,
    pub key: String,
    pub value: String,
}

#[derive(serde::Deserialize)]
pub struct BulkSetEnvVarRequest {
    pub container_name: String,
    pub vars: Vec<EnvVarItem>,
}

#[derive(serde::Deserialize)]
pub struct EnvVarItem {
    pub key: String,
    pub value: String,
}

#[derive(serde::Serialize)]
pub struct StackEnvVarResponse {
    pub id: String,
    pub stack_id: String,
    pub container_name: String,
    pub key: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<StackEnvVar> for StackEnvVarResponse {
    fn from(var: StackEnvVar) -> Self {
        Self {
            id: var.id,
            stack_id: var.stack_id,
            container_name: var.container_name,
            key: var.key,
            value: var.value,
            created_at: var.created_at,
            updated_at: var.updated_at,
        }
    }
}

pub struct EnvironmentService {
    db: SqlitePool,
}

impl EnvironmentService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn list(&self, stack_id: &str) -> Result<Vec<StackEnvVar>> {
        let vars = sqlx::query_as::<_, StackEnvVar>(
            "SELECT * FROM stack_env_vars WHERE stack_id = ? ORDER BY container_name, key",
        )
        .bind(stack_id)
        .fetch_all(&self.db)
        .await?;

        Ok(vars)
    }

    pub async fn get_env_map_for_container(
        &self,
        stack_id: &str,
        container_name: &str,
    ) -> Result<std::collections::HashMap<String, String>> {
        let vars = sqlx::query_as::<_, StackEnvVar>(
            "SELECT * FROM stack_env_vars WHERE stack_id = ? AND container_name = ?",
        )
        .bind(stack_id)
        .bind(container_name)
        .fetch_all(&self.db)
        .await?;

        let mut map = std::collections::HashMap::new();
        for var in vars {
            map.insert(var.key, var.value);
        }

        Ok(map)
    }

    pub async fn set(
        &self,
        stack_id: &str,
        container_name: &str,
        key: &str,
        value: &str,
    ) -> Result<StackEnvVar> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let existing = sqlx::query_as::<_, StackEnvVar>(
            "SELECT * FROM stack_env_vars WHERE stack_id = ? AND container_name = ? AND key = ?",
        )
        .bind(stack_id)
        .bind(container_name)
        .bind(key)
        .fetch_optional(&self.db)
        .await?;

        if let Some(var) = existing {
            sqlx::query("UPDATE stack_env_vars SET value = ?, updated_at = ? WHERE id = ?")
                .bind(value)
                .bind(&now)
                .bind(&var.id)
                .execute(&self.db)
                .await?;

            self.get(&var.id).await
        } else {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO stack_env_vars (id, stack_id, container_name, key, value, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&id)
            .bind(stack_id)
            .bind(container_name)
            .bind(key)
            .bind(value)
            .bind(&now)
            .bind(&now)
            .execute(&self.db)
            .await?;

            self.get(&id).await
        }
    }

    pub async fn get(&self, id: &str) -> Result<StackEnvVar> {
        sqlx::query_as::<_, StackEnvVar>("SELECT * FROM stack_env_vars WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Environment variable not found".to_string()))
    }

    pub async fn delete(&self, stack_id: &str, container_name: &str, key: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM stack_env_vars WHERE stack_id = ? AND container_name = ? AND key = ?",
        )
        .bind(stack_id)
        .bind(container_name)
        .bind(key)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn bulk_set(
        &self,
        stack_id: &str,
        container_name: &str,
        vars: Vec<EnvVarItem>,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;

        for var in vars {
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let existing = sqlx::query_scalar::<_, String>(
                "SELECT id FROM stack_env_vars WHERE stack_id = ? AND container_name = ? AND key = ?"
            )
            .bind(stack_id)
            .bind(container_name)
            .bind(&var.key)
            .fetch_optional(&mut *tx)
            .await?;

            if let Some(id) = existing {
                sqlx::query("UPDATE stack_env_vars SET value = ?, updated_at = ? WHERE id = ?")
                    .bind(&var.value)
                    .bind(&now)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            } else {
                let id = Uuid::new_v4().to_string();
                sqlx::query("INSERT INTO stack_env_vars (id, stack_id, container_name, key, value, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
                    .bind(id)
                    .bind(stack_id)
                    .bind(container_name)
                    .bind(&var.key)
                    .bind(&var.value)
                    .bind(&now)
                    .bind(&now)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }
}
