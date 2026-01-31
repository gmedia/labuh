use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::models::resource::{ContainerResource, ResourceMetric};
use crate::domain::resource_repository::ResourceRepository;
use crate::error::Result;

pub struct SqliteResourceRepository {
    pool: SqlitePool,
}

impl SqliteResourceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ResourceRepository for SqliteResourceRepository {
    async fn get_resource_limits(
        &self,
        stack_id: &str,
        service_name: &str,
    ) -> Result<Option<ContainerResource>> {
        let resource = sqlx::query_as::<_, ContainerResource>(
            "SELECT * FROM container_resources WHERE stack_id = ? AND service_name = ?",
        )
        .bind(stack_id)
        .bind(service_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(resource)
    }

    async fn list_resource_limits_for_stack(
        &self,
        stack_id: &str,
    ) -> Result<Vec<ContainerResource>> {
        let resources = sqlx::query_as::<_, ContainerResource>(
            "SELECT * FROM container_resources WHERE stack_id = ?",
        )
        .bind(stack_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(resources)
    }

    async fn update_resource_limits(
        &self,
        stack_id: &str,
        service_name: &str,
        cpu: Option<f64>,
        memory: Option<i64>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO container_resources (id, stack_id, service_name, cpu_limit, memory_limit, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(stack_id, service_name) DO UPDATE SET
                cpu_limit = excluded.cpu_limit,
                memory_limit = excluded.memory_limit,
                updated_at = excluded.updated_at
            "#
        )
        .bind(Uuid::new_v4().to_string())
        .bind(stack_id)
        .bind(service_name)
        .bind(cpu)
        .bind(memory)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save_metric(&self, metric: ResourceMetric) -> Result<()> {
        sqlx::query(
            "INSERT INTO resource_metrics (id, container_id, stack_id, cpu_usage, memory_usage, timestamp) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&metric.id)
        .bind(&metric.container_id)
        .bind(&metric.stack_id)
        .bind(metric.cpu_usage)
        .bind(metric.memory_usage)
        .bind(&metric.timestamp)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_metrics_for_stack(
        &self,
        stack_id: &str,
        since: &str,
    ) -> Result<Vec<ResourceMetric>> {
        let metrics = sqlx::query_as::<_, ResourceMetric>(
            "SELECT * FROM resource_metrics WHERE stack_id = ? AND timestamp >= ? ORDER BY timestamp ASC"
        )
        .bind(stack_id)
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        Ok(metrics)
    }

    async fn prune_metrics(&self, older_than: &str) -> Result<()> {
        sqlx::query("DELETE FROM resource_metrics WHERE timestamp < ?")
            .bind(older_than)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
