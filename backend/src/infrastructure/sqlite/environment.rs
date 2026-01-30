use crate::domain::environment_repository::EnvironmentRepository;
use crate::domain::models::environment::StackEnvVar;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;

pub struct SqliteEnvironmentRepository {
    pool: SqlitePool,
}

impl SqliteEnvironmentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EnvironmentRepository for SqliteEnvironmentRepository {
    async fn list_by_stack(&self, stack_id: &str) -> Result<Vec<StackEnvVar>> {
        let vars = sqlx::query_as::<_, StackEnvVar>(
            "SELECT * FROM stack_env_vars WHERE stack_id = ? ORDER BY container_name, key",
        )
        .bind(stack_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(vars)
    }

    async fn find_by_id(&self, id: &str) -> Result<StackEnvVar> {
        let var = sqlx::query_as::<_, StackEnvVar>("SELECT * FROM stack_env_vars WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Environment variable not found".to_string()))?;

        Ok(var)
    }

    async fn find_existing(
        &self,
        stack_id: &str,
        container_name: &str,
        key: &str,
    ) -> Result<Option<StackEnvVar>> {
        let var = sqlx::query_as::<_, StackEnvVar>(
            "SELECT * FROM stack_env_vars WHERE stack_id = ? AND container_name = ? AND key = ?",
        )
        .bind(stack_id)
        .bind(container_name)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(var)
    }

    async fn save(&self, var: StackEnvVar) -> Result<StackEnvVar> {
        // We can use an upsert or check and insert/update
        // For simplicity and since we have find_existing, we'll do manual check in usecase or just sqlx insert/update
        let existing = self
            .find_existing(&var.stack_id, &var.container_name, &var.key)
            .await?;

        if let Some(existing) = existing {
            sqlx::query(
                "UPDATE stack_env_vars SET value = ?, is_secret = ?, updated_at = ? WHERE id = ?",
            )
            .bind(&var.value)
            .bind(var.is_secret)
            .bind(&var.updated_at)
            .bind(&existing.id)
            .execute(&self.pool)
            .await?;

            self.find_by_id(&existing.id).await
        } else {
            sqlx::query(
                "INSERT INTO stack_env_vars (id, stack_id, container_name, key, value, is_secret, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&var.id)
            .bind(&var.stack_id)
            .bind(&var.container_name)
            .bind(&var.key)
            .bind(&var.value)
            .bind(var.is_secret)
            .bind(&var.created_at)
            .bind(&var.updated_at)
            .execute(&self.pool)
            .await?;

            Ok(var)
        }
    }

    async fn delete(&self, stack_id: &str, container_name: &str, key: &str) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM stack_env_vars WHERE stack_id = ? AND container_name = ? AND key = ?",
        )
        .bind(stack_id)
        .bind(container_name)
        .bind(key)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "Environment variable not found".to_string(),
            ));
        }

        Ok(())
    }
}
