#![allow(dead_code)]
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::deployment_log_repository::DeploymentLogRepository;
use crate::domain::models::deployment_log::{DeploymentLog, DeploymentLogResponse};
use crate::error::Result;

pub struct DeploymentLogUsecase {
    repo: Arc<dyn DeploymentLogRepository>,
}

impl DeploymentLogUsecase {
    pub fn new(repo: Arc<dyn DeploymentLogRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_logs(
        &self,
        stack_id: &str,
        limit: i32,
    ) -> Result<Vec<DeploymentLogResponse>> {
        let logs = self.repo.list_by_stack(stack_id, limit).await?;
        Ok(logs.into_iter().map(Into::into).collect())
    }

    pub async fn get_log(&self, id: &str) -> Result<DeploymentLogResponse> {
        let log = self.repo.find_by_id(id).await?;
        Ok(log.into())
    }

    pub async fn create_log(
        &self,
        stack_id: &str,
        trigger_type: &str,
    ) -> Result<DeploymentLogResponse> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let log = DeploymentLog {
            id,
            stack_id: stack_id.to_string(),
            trigger_type: trigger_type.to_string(),
            status: "pending".to_string(),
            logs: None,
            started_at: now,
            finished_at: None,
        };

        let saved = self.repo.save(log).await?;
        Ok(saved.into())
    }

    pub async fn update_status(
        &self,
        id: &str,
        status: &str,
        logs: Option<&str>,
    ) -> Result<DeploymentLogResponse> {
        let mut log = self.repo.find_by_id(id).await?;
        log.status = status.to_string();
        log.logs = logs.map(|s| s.to_string());
        log.finished_at = Some(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

        let saved = self.repo.save(log).await?;
        Ok(saved.into())
    }

    pub async fn append_log(&self, id: &str, log_line: &str) -> Result<()> {
        self.repo.append_log(id, log_line).await
    }
}
