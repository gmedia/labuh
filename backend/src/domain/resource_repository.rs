use crate::domain::models::resource::{ContainerResource, ResourceMetric};
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ResourceRepository: Send + Sync {
    // Resource limits
    async fn get_resource_limits(
        &self,
        stack_id: &str,
        service_name: &str,
    ) -> Result<Option<ContainerResource>>;
    async fn list_resource_limits_for_stack(
        &self,
        stack_id: &str,
    ) -> Result<Vec<ContainerResource>>;
    async fn update_resource_limits(
        &self,
        stack_id: &str,
        service_name: &str,
        cpu: Option<f64>,
        memory: Option<i64>,
    ) -> Result<()>;

    // Metrics
    async fn save_metric(&self, metric: ResourceMetric) -> Result<()>;
    async fn get_metrics_for_stack(
        &self,
        stack_id: &str,
        since: &str,
    ) -> Result<Vec<ResourceMetric>>;
    async fn prune_metrics(&self, older_than: &str) -> Result<()>;
}
