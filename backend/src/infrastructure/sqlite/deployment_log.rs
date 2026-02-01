use crate::domain::deployment_log_repository::DeploymentLogRepository;
use crate::domain::models::DeploymentLog;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;

pub struct SqliteDeploymentLogRepository {
    pool: SqlitePool,
}

impl SqliteDeploymentLogRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeploymentLogRepository for SqliteDeploymentLogRepository {
    async fn list_by_stack(&self, stack_id: &str, limit: i32) -> Result<Vec<DeploymentLog>> {
        let logs = sqlx::query_as::<_, DeploymentLog>(
            "SELECT * FROM deployment_logs WHERE stack_id = ? ORDER BY started_at DESC LIMIT ?",
        )
        .bind(stack_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    async fn find_by_id(&self, id: &str) -> Result<DeploymentLog> {
        let log = sqlx::query_as::<_, DeploymentLog>("SELECT * FROM deployment_logs WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Deployment log not found".to_string()))?;

        Ok(log)
    }

    async fn save(&self, log: DeploymentLog) -> Result<DeploymentLog> {
        let existing = sqlx::query("SELECT id FROM deployment_logs WHERE id = ?")
            .bind(&log.id)
            .fetch_optional(&self.pool)
            .await?;

        if existing.is_some() {
            sqlx::query(
                "UPDATE deployment_logs SET status = ?, logs = ?, finished_at = ? WHERE id = ?",
            )
            .bind(&log.status)
            .bind(&log.logs)
            .bind(&log.finished_at)
            .bind(&log.id)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                "INSERT INTO deployment_logs (id, stack_id, trigger_type, status, logs, started_at, finished_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&log.id)
            .bind(&log.stack_id)
            .bind(&log.trigger_type)
            .bind(&log.status)
            .bind(&log.logs)
            .bind(&log.started_at)
            .bind(&log.finished_at)
            .execute(&self.pool)
            .await?;
        }

        Ok(log)
    }
}
