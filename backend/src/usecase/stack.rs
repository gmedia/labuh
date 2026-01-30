use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::compose::{parse_compose, service_to_container_request};
use crate::domain::models::Stack;
use crate::domain::runtime::RuntimePort;
use crate::domain::stack_repository::StackRepository;
use crate::error::Result;
use crate::usecase::environment::EnvironmentUsecase;

pub struct StackUsecase {
    repo: Arc<dyn StackRepository>,
    runtime: Arc<dyn RuntimePort>,
    environment_usecase: Arc<EnvironmentUsecase>,
}

impl StackUsecase {
    pub fn new(
        repo: Arc<dyn StackRepository>,
        runtime: Arc<dyn RuntimePort>,
        environment_usecase: Arc<EnvironmentUsecase>,
    ) -> Self {
        Self {
            repo,
            runtime,
            environment_usecase,
        }
    }

    pub async fn list_stacks(&self, user_id: &str) -> Result<Vec<Stack>> {
        self.repo.list_by_user(user_id).await
    }

    pub async fn get_stack(&self, id: &str, user_id: &str) -> Result<Stack> {
        self.repo.find_by_id(id, user_id).await
    }

    pub async fn create_stack(
        &self,
        name: &str,
        compose_content: &str,
        user_id: &str,
    ) -> Result<Stack> {
        let parsed = parse_compose(compose_content)?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let stack = Stack {
            id: id.clone(),
            name: name.to_string(),
            user_id: user_id.to_string(),
            compose_content: Some(compose_content.to_string()),
            status: "creating".to_string(),
            webhook_token: Some(token),
            created_at: now.clone(),
            updated_at: now,
        };

        self.repo.create(stack.clone()).await?;

        for service in &parsed.services {
            let mut config = service_to_container_request(service, &id, name);

            let db_env = self
                .environment_usecase
                .get_env_map_for_container(&id, &service.name)
                .await
                .unwrap_or_default();

            if !db_env.is_empty() {
                let mut merged_env = config.env.unwrap_or_default();
                for (key, value) in &db_env {
                    let entry = format!("{}={}", key, value);
                    if let Some(pos) = merged_env
                        .iter()
                        .position(|e: &String| e.starts_with(&format!("{}=", key)))
                    {
                        merged_env[pos] = entry;
                    } else {
                        merged_env.push(entry);
                    }
                }
                config.env = Some(merged_env);
            }

            self.runtime.pull_image(&config.image).await?;
            self.runtime.create_container(config).await?;
        }

        self.repo.update_status(&id, "stopped").await?;
        self.get_stack(&id, user_id).await
    }

    pub async fn start_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;

        for container in containers {
            if container.state != "running" {
                self.runtime.start_container(&container.id).await?;
            }
        }

        self.repo.update_status(id, "running").await?;
        Ok(())
    }

    pub async fn stop_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;

        for container in containers {
            if container.state == "running" {
                self.runtime.stop_container(&container.id).await?;
            }
        }

        self.repo.update_status(id, "stopped").await?;
        Ok(())
    }

    pub async fn redeploy_stack(&self, id: &str) -> Result<()> {
        let stack = self.repo.find_by_id_internal(id).await?;
        let compose_content = stack.compose_content.clone().ok_or_else(|| {
            crate::error::AppError::BadRequest("Stack has no compose content".to_string())
        })?;

        self.repo.update_status(id, "deploying").await?;
        let parsed = parse_compose(&compose_content)?;

        for service in &parsed.services {
            let mut config = service_to_container_request(service, &stack.id, &stack.name);

            let db_env = self
                .environment_usecase
                .get_env_map_for_container(id, &service.name)
                .await
                .unwrap_or_default();

            if !db_env.is_empty() {
                let mut merged_env = config.env.unwrap_or_default();
                for (key, value) in &db_env {
                    let entry = format!("{}={}", key, value);
                    if let Some(pos) = merged_env
                        .iter()
                        .position(|e: &String| e.starts_with(&format!("{}=", key)))
                    {
                        merged_env[pos] = entry;
                    } else {
                        merged_env.push(entry);
                    }
                }
                config.env = Some(merged_env);
            }

            self.runtime.pull_image(&config.image).await?;

            let containers = self.get_stack_containers(&stack.id, &stack.name).await?;
            let prefix = format!("/{}-{}", stack.name, service.name);
            for c in containers {
                if c.names.iter().any(|n| n == &prefix) {
                    let _ = self.runtime.stop_container(&c.id).await;
                    let _ = self.runtime.remove_container(&c.id, true).await;
                }
            }

            self.runtime.create_container(config).await?;
        }

        self.start_stack(id, &stack.user_id).await?;
        Ok(())
    }

    pub async fn remove_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;

        for container in containers {
            let _ = self.runtime.stop_container(&container.id).await;
            let _ = self.runtime.remove_container(&container.id, true).await;
        }

        self.repo.delete(id).await?;
        Ok(())
    }

    pub async fn get_stack_health(
        &self,
        id: &str,
        user_id: &str,
    ) -> Result<crate::domain::models::stack::StackHealth> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;

        let total = containers.len();
        let running = containers.iter().filter(|c| c.state == "running").count();
        let stopped = containers
            .iter()
            .filter(|c| c.state == "exited" || c.state == "created")
            .count();
        let unhealthy = containers
            .iter()
            .filter(|c| c.state != "running" && c.state != "exited" && c.state != "created")
            .count();

        let status = if total == 0 {
            "empty".to_string()
        } else if running == total {
            "healthy".to_string()
        } else if running > 0 {
            "partial".to_string()
        } else {
            "stopped".to_string()
        };

        Ok(crate::domain::models::stack::StackHealth {
            status,
            total,
            running,
            stopped,
            unhealthy,
            containers: containers
                .into_iter()
                .map(|c| crate::domain::models::stack::ContainerHealth {
                    id: c.id,
                    name: c.names.first().cloned().unwrap_or_default(),
                    state: c.state,
                    status: c.status,
                })
                .collect(),
        })
    }

    pub async fn get_stack_logs(
        &self,
        id: &str,
        user_id: &str,
        tail: Option<usize>,
    ) -> Result<Vec<crate::domain::models::stack::StackLogEntry>> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;

        let mut all_logs = Vec::new();
        let tail_count = tail.unwrap_or(100);

        for container in containers {
            let container_name = container
                .names
                .first()
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_else(|| container.id.clone());
            match self.runtime.get_logs(&container.id, tail_count).await {
                Ok(logs) => {
                    for line in logs {
                        all_logs.push(crate::domain::models::stack::StackLogEntry {
                            container: container_name.clone(),
                            message: line,
                        });
                    }
                }
                Err(e) => {
                    all_logs.push(crate::domain::models::stack::StackLogEntry {
                        container: container_name.clone(),
                        message: format!("[error fetching logs: {}]", e),
                    });
                }
            }
        }
        Ok(all_logs)
    }

    pub async fn update_stack_compose(
        &self,
        id: &str,
        compose_content: &str,
        user_id: &str,
    ) -> Result<()> {
        let _stack = self.get_stack(id, user_id).await?;
        parse_compose(compose_content)?;
        self.repo.update_compose(id, compose_content).await?;
        self.redeploy_stack(id).await?;
        Ok(())
    }

    pub async fn regenerate_webhook_token(&self, id: &str, user_id: &str) -> Result<String> {
        let _stack = self.get_stack(id, user_id).await?;
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        self.repo.update_webhook_token(id, &token).await?;
        Ok(token)
    }

    pub async fn redeploy_service(
        &self,
        stack_id: &str,
        service_name: &str,
        user_id: &str,
    ) -> Result<()> {
        let stack = self.get_stack(stack_id, user_id).await?;
        let compose_content = stack.compose_content.ok_or_else(|| {
            crate::error::AppError::BadRequest("Stack has no compose content".to_string())
        })?;
        let parsed = parse_compose(&compose_content)?;
        let service = parsed
            .services
            .iter()
            .find(|s| {
                s.name.to_lowercase() == service_name.to_lowercase()
                    || format!("{}-{}", stack.name, s.name).to_lowercase()
                        == service_name.to_lowercase()
            })
            .ok_or_else(|| {
                crate::error::AppError::NotFound(format!("Service {} not found", service_name))
            })?;
        let mut config = service_to_container_request(service, &stack.id, &stack.name);

        let db_env = self
            .environment_usecase
            .get_env_map_for_container(stack_id, &service.name)
            .await
            .unwrap_or_default();

        if !db_env.is_empty() {
            let mut merged_env = config.env.unwrap_or_default();
            for (key, value) in &db_env {
                let entry = format!("{}={}", key, value);
                if let Some(pos) = merged_env
                    .iter()
                    .position(|e: &String| e.starts_with(&format!("{}=", key)))
                {
                    merged_env[pos] = entry;
                } else {
                    merged_env.push(entry);
                }
            }
            config.env = Some(merged_env);
        }

        self.runtime.pull_image(&config.image).await?;
        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;
        let prefix = format!("/{}-{}", stack.name, service.name);
        for c in containers {
            if c.names.iter().any(|n| n == &prefix) {
                let _ = self.runtime.stop_container(&c.id).await;
                let _ = self.runtime.remove_container(&c.id, true).await;
            }
        }
        self.runtime.create_container(config).await?;
        Ok(())
    }

    pub async fn validate_webhook_token(&self, id: &str, token: &str) -> Result<Stack> {
        self.repo.validate_webhook_token(id, token).await
    }

    async fn get_stack_containers(
        &self,
        _stack_id: &str,
        stack_name: &str,
    ) -> Result<Vec<crate::domain::runtime::ContainerInfo>> {
        let all = self.runtime.list_containers(true).await?;
        let prefix = format!("/{}-", stack_name);
        Ok(all
            .into_iter()
            .filter(|c| c.names.iter().any(|n| n.starts_with(&prefix)))
            .collect())
    }
}
