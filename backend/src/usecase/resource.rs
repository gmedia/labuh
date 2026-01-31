use crate::domain::models::resource::{ContainerResource, ResourceMetric};
use crate::domain::resource_repository::ResourceRepository;
use crate::domain::stack_repository::StackRepository;
use crate::error::Result;
use chrono::{Duration, Utc};
use std::sync::Arc;

use crate::domain::TeamRepository;
use crate::error::AppError;

pub struct ResourceUsecase {
    repo: Arc<dyn ResourceRepository>,
    stack_repo: Arc<dyn StackRepository>,
    team_repo: Arc<dyn TeamRepository>,
}

impl ResourceUsecase {
    pub fn new(
        repo: Arc<dyn ResourceRepository>,
        stack_repo: Arc<dyn StackRepository>,
        team_repo: Arc<dyn TeamRepository>,
    ) -> Self {
        Self {
            repo,
            stack_repo,
            team_repo,
        }
    }

    pub async fn update_limits(
        &self,
        stack_id: &str,
        service_name: &str,
        user_id: &str,
        cpu: Option<f64>,
        memory: Option<i64>,
    ) -> Result<()> {
        // Verify ownership via team
        let stack = self.stack_repo.find_by_id_internal(stack_id).await?;
        let _role = self
            .team_repo
            .get_user_role(&stack.team_id, user_id)
            .await?
            .ok_or(AppError::Forbidden("Access denied".to_string()))?;

        self.repo
            .update_resource_limits(stack_id, service_name, cpu, memory)
            .await?;
        Ok(())
    }

    pub async fn get_limits(
        &self,
        stack_id: &str,
        user_id: &str,
    ) -> Result<Vec<ContainerResource>> {
        // Verify ownership
        let stack = self.stack_repo.find_by_id_internal(stack_id).await?;
        let _role = self
            .team_repo
            .get_user_role(&stack.team_id, user_id)
            .await?
            .ok_or(AppError::Forbidden("Access denied".to_string()))?;

        self.repo.list_resource_limits_for_stack(stack_id).await
    }

    pub async fn get_metrics(
        &self,
        stack_id: &str,
        user_id: &str,
        range: &str,
    ) -> Result<Vec<ResourceMetric>> {
        // Verify ownership
        let stack = self.stack_repo.find_by_id_internal(stack_id).await?;
        let _role = self
            .team_repo
            .get_user_role(&stack.team_id, user_id)
            .await?
            .ok_or(AppError::Forbidden("Access denied".to_string()))?;

        let duration = match range {
            "1h" => Duration::hours(1),
            "6h" => Duration::hours(6),
            "24h" => Duration::hours(24),
            "7d" => Duration::days(7),
            "30d" => Duration::days(30),
            _ => Duration::hours(1),
        };

        let since = (Utc::now() - duration).to_rfc3339();
        self.repo.get_metrics_for_stack(stack_id, &since).await
    }
}
