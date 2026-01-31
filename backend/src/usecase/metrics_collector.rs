use chrono::Utc;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::domain::models::resource::ResourceMetric;
use crate::domain::resource_repository::ResourceRepository;
use crate::domain::runtime::RuntimePort;
use crate::domain::stack_repository::StackRepository;

pub struct MetricsCollector {
    stack_repo: Arc<dyn StackRepository>,
    resource_repo: Arc<dyn ResourceRepository>,
    runtime: Arc<dyn RuntimePort>,
}

impl MetricsCollector {
    pub fn new(
        stack_repo: Arc<dyn StackRepository>,
        resource_repo: Arc<dyn ResourceRepository>,
        runtime: Arc<dyn RuntimePort>,
    ) -> Self {
        Self {
            stack_repo,
            resource_repo,
            runtime,
        }
    }

    pub async fn start(&self) {
        tracing::info!("Starting metrics collector...");

        loop {
            if let Err(e) = self.collect_metrics().await {
                tracing::error!("Error collecting metrics: {}", e);
            }

            // Collect every minute
            sleep(Duration::from_secs(60)).await;
        }
    }

    async fn collect_metrics(&self) -> crate::error::Result<()> {
        let stacks = self.stack_repo.list_all().await?;

        for stack in stacks {
            let containers = match self.runtime.list_containers(false).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Failed to list containers for metrics: {}", e);
                    continue;
                }
            };

            let stack_containers: Vec<_> = containers
                .into_iter()
                .filter(|c| {
                    c.labels
                        .get("labuh.stack.id")
                        .map(|id| id == &stack.id)
                        .unwrap_or(false)
                })
                .collect();

            for container in stack_containers {
                match self.runtime.get_stats(&container.id).await {
                    Ok(stats) => {
                        let metric = ResourceMetric {
                            id: Uuid::new_v4().to_string(),
                            container_id: container.id.clone(),
                            stack_id: stack.id.clone(),
                            cpu_usage: stats.cpu_percent,
                            memory_usage: stats.memory_usage as i64,
                            timestamp: Utc::now().to_rfc3339(),
                        };

                        if let Err(e) = self.resource_repo.save_metric(metric).await {
                            tracing::error!("Failed to save metric: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::debug!(
                            "Failed to get stats for container {}: {}",
                            container.id,
                            e
                        );
                    }
                }
            }
        }

        // Also prune old metrics
        if let Err(e) = self
            .resource_repo
            .prune_metrics(&(Utc::now() - chrono::Duration::days(30)).to_rfc3339())
            .await
        {
            tracing::error!("Failed to prune metrics: {}", e);
        }

        Ok(())
    }
}
