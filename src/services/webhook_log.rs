//! Webhook log service for tracking webhook invocations

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::Result;

/// Webhook log entry model
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct WebhookLog {
    pub id: String,
    pub stack_id: String,
    pub trigger_type: String,
    pub status: String,
    pub payload: Option<String>,
    pub response: Option<String>,
    pub triggered_at: String,
    pub completed_at: Option<String>,
}

pub struct WebhookLogService {
    db: SqlitePool,
}

impl WebhookLogService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Create a new webhook log entry (initially pending)
    pub async fn create(&self, stack_id: &str, trigger_type: &str, payload: Option<&str>) -> Result<WebhookLog> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO webhook_logs (id, stack_id, trigger_type, status, payload, triggered_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(stack_id)
        .bind(trigger_type)
        .bind("pending")
        .bind(payload)
        .bind(&now)
        .execute(&self.db)
        .await?;

        self.get(&id).await
    }

    /// Get a webhook log by ID
    pub async fn get(&self, id: &str) -> Result<WebhookLog> {
        let log = sqlx::query_as::<_, WebhookLog>(
            "SELECT * FROM webhook_logs WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        Ok(log)
    }

    /// Mark webhook as completed (success or failed)
    pub async fn complete(&self, id: &str, success: bool, response: Option<&str>) -> Result<WebhookLog> {
        let now = Utc::now().to_rfc3339();
        let status = if success { "success" } else { "failed" };

        sqlx::query(
            "UPDATE webhook_logs SET status = ?, response = ?, completed_at = ? WHERE id = ?"
        )
        .bind(status)
        .bind(response)
        .bind(&now)
        .bind(id)
        .execute(&self.db)
        .await?;

        self.get(id).await
    }

    /// List webhook logs for a stack
    pub async fn list(&self, stack_id: &str, limit: Option<usize>) -> Result<Vec<WebhookLog>> {
        let limit = limit.unwrap_or(50);

        let logs = sqlx::query_as::<_, WebhookLog>(
            "SELECT * FROM webhook_logs WHERE stack_id = ? ORDER BY triggered_at DESC LIMIT ?"
        )
        .bind(stack_id)
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        Ok(logs)
    }
}
