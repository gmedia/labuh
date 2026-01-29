//! Environment service for managing stack environment variables

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, Result};

/// Stack environment variable model
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct StackEnvVar {
    pub id: String,
    pub stack_id: String,
    pub container_name: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Response type that masks secret values
#[derive(Debug, Clone, serde::Serialize)]
pub struct StackEnvVarResponse {
    pub id: String,
    pub stack_id: String,
    pub container_name: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<StackEnvVar> for StackEnvVarResponse {
    fn from(env: StackEnvVar) -> Self {
        Self {
            id: env.id,
            stack_id: env.stack_id,
            container_name: env.container_name,
            key: env.key,
            value: if env.is_secret {
                "********".to_string()
            } else {
                env.value
            },
            is_secret: env.is_secret,
            created_at: env.created_at,
            updated_at: env.updated_at,
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

    /// List all environment variables for a stack
    pub async fn list(&self, stack_id: &str) -> Result<Vec<StackEnvVar>> {
        let vars = sqlx::query_as::<_, StackEnvVar>(
            "SELECT * FROM stack_env_vars WHERE stack_id = ? ORDER BY container_name, key",
        )
        .bind(stack_id)
        .fetch_all(&self.db)
        .await?;

        Ok(vars)
    }

    /// Get environment variables as a HashMap (for container creation)
    /// Merges global and container-specific variables
    pub async fn get_env_map_for_container(
        &self,
        stack_id: &str,
        container_name: &str,
    ) -> Result<std::collections::HashMap<String, String>> {
        // Fetch all vars for this stack
        let vars = self.list(stack_id).await?;

        let mut map = std::collections::HashMap::new();

        // 1. Apply global vars (container_name is empty)
        for v in vars.iter().filter(|v| v.container_name.is_empty()) {
            map.insert(v.key.clone(), v.value.clone());
        }

        // 2. Override with container-specific vars
        for v in vars.iter().filter(|v| v.container_name == container_name) {
            map.insert(v.key.clone(), v.value.clone());
        }

        Ok(map)
    }

    /// Set an environment variable (create or update)
    pub async fn set(
        &self,
        stack_id: &str,
        container_name: &str,
        key: &str,
        value: &str,
        is_secret: bool,
    ) -> Result<StackEnvVar> {
        let now = Utc::now().to_rfc3339();

        // Try to find existing
        let existing = sqlx::query_as::<_, StackEnvVar>(
            "SELECT * FROM stack_env_vars WHERE stack_id = ? AND container_name = ? AND key = ?",
        )
        .bind(stack_id)
        .bind(container_name)
        .bind(key)
        .fetch_optional(&self.db)
        .await?;

        if let Some(env) = existing {
            // Update existing
            sqlx::query(
                "UPDATE stack_env_vars SET value = ?, is_secret = ?, updated_at = ? WHERE id = ?",
            )
            .bind(value)
            .bind(is_secret)
            .bind(&now)
            .bind(&env.id)
            .execute(&self.db)
            .await?;

            return self.get(&env.id).await;
        }

        // Create new
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO stack_env_vars (id, stack_id, container_name, key, value, is_secret, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(stack_id)
        .bind(container_name)
        .bind(key)
        .bind(value)
        .bind(is_secret)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        self.get(&id).await
    }

    /// Get a single environment variable by ID
    pub async fn get(&self, id: &str) -> Result<StackEnvVar> {
        sqlx::query_as::<_, StackEnvVar>("SELECT * FROM stack_env_vars WHERE id = ?")
            .bind(id)
            .fetch_one(&self.db)
            .await
            .map_err(|_| AppError::NotFound("Environment variable not found".to_string()))
    }

    /// Delete an environment variable
    pub async fn delete(&self, stack_id: &str, container_name: &str, key: &str) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM stack_env_vars WHERE stack_id = ? AND container_name = ? AND key = ?",
        )
        .bind(stack_id)
        .bind(container_name)
        .bind(key)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Environment variable '{}' for container '{}' not found",
                key, container_name
            )));
        }

        Ok(())
    }

    /// Bulk set environment variables
    pub async fn bulk_set(
        &self,
        stack_id: &str,
        container_name: &str,
        vars: Vec<(String, String, bool)>,
    ) -> Result<Vec<StackEnvVar>> {
        let mut results = Vec::new();
        for (key, value, is_secret) in vars {
            let var = self
                .set(stack_id, container_name, &key, &value, is_secret)
                .await?;
            results.push(var);
        }
        Ok(results)
    }
}
