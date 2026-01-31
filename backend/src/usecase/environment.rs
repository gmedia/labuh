use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::environment_repository::EnvironmentRepository;
use crate::domain::models::environment::{StackEnvVar, StackEnvVarResponse};
use crate::error::Result;

pub struct EnvironmentUsecase {
    repo: Arc<dyn EnvironmentRepository>,
}

impl EnvironmentUsecase {
    pub fn new(repo: Arc<dyn EnvironmentRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_vars(&self, stack_id: &str) -> Result<Vec<StackEnvVarResponse>> {
        let vars = self.repo.list_by_stack(stack_id).await?;
        Ok(vars.into_iter().map(Into::into).collect())
    }

    pub async fn get_raw_vars(&self, stack_id: &str) -> Result<Vec<StackEnvVar>> {
        self.repo.list_by_stack(stack_id).await
    }

    pub async fn get_env_map_for_container(
        &self,
        stack_id: &str,
        container_name: &str,
    ) -> Result<HashMap<String, String>> {
        let vars = self.repo.list_by_stack(stack_id).await?;
        let mut map = HashMap::new();

        // Global vars
        for v in vars.iter().filter(|v| v.container_name.is_empty()) {
            map.insert(v.key.clone(), v.value.clone());
        }

        // Container-specific overrides
        for v in vars.iter().filter(|v| v.container_name == container_name) {
            map.insert(v.key.clone(), v.value.clone());
        }

        Ok(map)
    }

    pub async fn set_var(
        &self,
        stack_id: &str,
        container_name: &str,
        key: &str,
        value: &str,
        is_secret: bool,
    ) -> Result<StackEnvVarResponse> {
        let now = Utc::now().to_rfc3339();
        let var = StackEnvVar {
            id: Uuid::new_v4().to_string(),
            stack_id: stack_id.to_string(),
            container_name: container_name.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            is_secret,
            created_at: now.clone(),
            updated_at: now,
        };

        let saved = self.repo.save(var).await?;
        Ok(saved.into())
    }

    pub async fn bulk_set(
        &self,
        stack_id: &str,
        container_name: &str,
        vars: Vec<(String, String, bool)>,
    ) -> Result<Vec<StackEnvVarResponse>> {
        let mut results = Vec::new();
        for (key, value, is_secret) in vars {
            results.push(
                self.set_var(stack_id, container_name, &key, &value, is_secret)
                    .await?,
            );
        }
        Ok(results)
    }

    pub async fn delete_var(&self, stack_id: &str, container_name: &str, key: &str) -> Result<()> {
        self.repo.delete(stack_id, container_name, key).await
    }
}
