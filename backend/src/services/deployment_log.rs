#![allow(dead_code)]
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::models::DeploymentLog;
use crate::error::Result;

#[derive(serde::Deserialize)]
pub struct CreateDeploymentLog {
    pub stack_id: String,
    pub trigger_type: String,
}

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

        let log = DeploymentLog {
            id: id.clone(),
            stack_id: input.stack_id,
            trigger_type: input.trigger_type,
            status: "pending".to_string(),
            logs: None,
            started_at: now,
            finished_at: None,
        };

        sqlx::query(
            r#"
            INSERT INTO deployment_logs (id, stack_id, trigger_type, status, logs, started_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&log.id)
        .bind(&log.stack_id)
        .bind(&log.trigger_type)
        .bind(&log.status)
        .bind(&log.logs)
        .bind(&log.started_at)
        .execute(&self.pool)
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
            r#"
            UPDATE deployment_logs
            SET status = ?, logs = ?, finished_at = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(logs)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        let log = sqlx::query_as::<_, DeploymentLog>("SELECT * FROM deployment_logs WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(log)
    }
}
