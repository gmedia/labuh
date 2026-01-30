use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::Result;
use crate::models::deployment_log::{CreateDeploymentLog, DeploymentLog};

pub struct DeploymentLogService {
    pool: SqlitePool,
}

impl DeploymentLogService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: CreateDeploymentLog) -> Result<DeploymentLog> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        sqlx::query(
            r#"
            INSERT INTO deployment_logs (id, stack_id, trigger_type, status, started_at)
            VALUES (?, ?, ?, 'pending', ?)
            "#,
        )
        .bind(&id)
        .bind(&input.stack_id)
        .bind(&input.trigger_type)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        self.get(&id).await
    }

    pub async fn get(&self, id: &str) -> Result<DeploymentLog> {
        let log = sqlx::query_as::<_, DeploymentLog>("SELECT * FROM deployment_logs WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(log)
    }

    pub async fn list_by_stack(&self, stack_id: &str, limit: i32) -> Result<Vec<DeploymentLog>> {
        let logs = sqlx::query_as::<_, DeploymentLog>(
            "SELECT * FROM deployment_logs WHERE stack_id = ? ORDER BY started_at DESC LIMIT ?",
        )
        .bind(stack_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    pub async fn update_status(
        &self,
        id: &str,
        status: &str,
        logs: Option<&str>,
    ) -> Result<DeploymentLog> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        sqlx::query(
            "UPDATE deployment_logs SET status = ?, logs = ?, finished_at = ? WHERE id = ?",
        )
        .bind(status)
        .bind(logs)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get(id).await
    }

    #[allow(dead_code)]
    pub async fn append_log(&self, id: &str, log_line: &str) -> Result<()> {
        let current = self.get(id).await?;
        let new_logs = match current.logs {
            Some(existing) => format!("{}\n{}", existing, log_line),
            None => log_line.to_string(),
        };

        sqlx::query("UPDATE deployment_logs SET logs = ? WHERE id = ?")
            .bind(&new_logs)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
