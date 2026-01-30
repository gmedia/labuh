#![allow(dead_code)]
use crate::domain::compose::{parse_compose, service_to_container_request};
use crate::domain::models::Stack;
use crate::error::Result;
use crate::services::container::ContainerService;
use crate::services::environment::EnvironmentService;
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

pub struct StackService {
    db: SqlitePool,
    container_service: Arc<ContainerService>,
    environment_service: Arc<EnvironmentService>,
}

impl StackService {
    pub fn new(
        db: SqlitePool,
        container_service: Arc<ContainerService>,
        environment_service: Arc<EnvironmentService>,
    ) -> Self {
        Self {
            db,
            container_service,
            environment_service,
        }
    }

    pub async fn list_stacks(&self, user_id: &str) -> Result<Vec<Stack>> {
        let stacks = sqlx::query_as::<_, Stack>("SELECT * FROM stacks WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(&self.db)
            .await?;
        Ok(stacks)
    }

    pub async fn get_stack(&self, id: &str, user_id: &str) -> Result<Stack> {
        let stack = sqlx::query_as::<_, Stack>("SELECT * FROM stacks WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Stack not found".to_string()))?;
        Ok(stack)
    }

    pub async fn create_stack(
        &self,
        name: &str,
        compose_content: &str,
        user_id: &str,
    ) -> Result<Stack> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let webhook_token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let stack = Stack {
            id: id.clone(),
            name: name.to_string(),
            compose_content: Some(compose_content.to_string()),
            status: "created".to_string(),
            webhook_token: Some(webhook_token),
            user_id: user_id.to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        sqlx::query(
            "INSERT INTO stacks (id, name, compose_content, status, webhook_token, user_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&stack.id)
        .bind(&stack.name)
        .bind(&stack.compose_content)
        .bind(&stack.status)
        .bind(&stack.webhook_token)
        .bind(&stack.user_id)
        .bind(&stack.created_at)
        .bind(&stack.updated_at)
        .execute(&self.db)
        .await?;

        Ok(stack)
    }

    pub async fn get_stack_containers(
        &self,
        _stack_id: &str,
        stack_name: &str,
    ) -> Result<Vec<crate::services::container::ContainerInfo>> {
        let all = self.container_service.list_containers(true).await?;
        let prefix = format!("/{}-", stack_name);
        Ok(all
            .into_iter()
            .filter(|c| c.names.iter().any(|n| n.starts_with(&prefix)))
            .collect())
    }

    pub async fn start_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;

        for container in containers {
            if container.state != "running" {
                self.container_service
                    .start_container(&container.id)
                    .await?;
            }
        }

        sqlx::query("UPDATE stacks SET status = 'running' WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn stop_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;

        for container in containers {
            if container.state == "running" {
                self.container_service.stop_container(&container.id).await?;
            }
        }

        sqlx::query("UPDATE stacks SET status = 'stopped' WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Stack> {
        let stack = sqlx::query_as::<_, Stack>("SELECT * FROM stacks WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Stack not found".to_string()))?;
        Ok(stack)
    }

    pub async fn validate_webhook_token(&self, stack_id: &str, token: &str) -> Result<Stack> {
        let stack =
            sqlx::query_as::<_, Stack>("SELECT * FROM stacks WHERE id = ? AND webhook_token = ?")
                .bind(stack_id)
                .bind(token)
                .fetch_optional(&self.db)
                .await?
                .ok_or_else(|| crate::error::AppError::InvalidCredentials)?;
        Ok(stack)
    }

    pub async fn regenerate_webhook_token(&self, id: &str, user_id: &str) -> Result<String> {
        let _ = self.get_stack(id, user_id).await?;
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        sqlx::query("UPDATE stacks SET webhook_token = ? WHERE id = ?")
            .bind(&token)
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(token)
    }

    pub async fn redeploy_stack(&self, id: &str) -> Result<()> {
        let stack = self.get(id).await?;
        let compose_content = stack.compose_content.ok_or_else(|| {
            crate::error::AppError::BadRequest("Stack has no compose content".to_string())
        })?;

        sqlx::query("UPDATE stacks SET status = 'deploying' WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;

        let parsed = parse_compose(&compose_content)?;

        for service in &parsed.services {
            let mut config = service_to_container_request(service, &stack.id, &stack.name);

            let db_env = self
                .environment_service
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

            self.container_service.pull_image(&config.image).await?;

            let containers = self.get_stack_containers(&stack.id, &stack.name).await?;
            let prefix = format!("/{}-{}", stack.name, service.name);
            for c in containers {
                if c.names.iter().any(|n| n == &prefix) {
                    let _ = self.container_service.stop_container(&c.id).await;
                    let _ = self.container_service.remove_container(&c.id, true).await;
                }
            }

            self.container_service
                .create_container(config.into())
                .await?;
        }

        self.start_stack(id, &stack.user_id).await?;
        Ok(())
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
            .environment_service
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

        self.container_service.pull_image(&config.image).await?;

        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;
        let prefix = format!("/{}-{}", stack.name, service.name);
        for c in containers {
            if c.names.iter().any(|n| n == &prefix) {
                let _ = self.container_service.stop_container(&c.id).await;
                let _ = self.container_service.remove_container(&c.id, true).await;
            }
        }

        self.container_service
            .create_container(config.into())
            .await?;
        // Start the single container
        let prefix = format!("/{}-{}", stack.name, service.name);
        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;
        if let Some(c) = containers
            .iter()
            .find(|c| c.names.iter().any(|n| n == &prefix))
        {
            self.container_service.start_container(&c.id).await?;
        }

        Ok(())
    }

    pub async fn remove_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;

        for container in containers {
            let _ = self.container_service.stop_container(&container.id).await;
            let _ = self
                .container_service
                .remove_container(&container.id, true)
                .await;
        }

        sqlx::query("DELETE FROM stacks WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn get_stack_health(&self, id: &str, user_id: &str) -> Result<StackHealth> {
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

        Ok(StackHealth {
            status,
            total,
            running,
            stopped,
            unhealthy,
            containers: containers
                .into_iter()
                .map(|c| ContainerHealth {
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
    ) -> Result<Vec<StackLogEntry>> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id, &stack.name).await?;

        let mut all_logs = Vec::new();
        for container in containers {
            let container_name = container.names.first().cloned().unwrap_or_default();
            let logs = self
                .container_service
                .get_container_logs(&container.id, tail.unwrap_or(100))
                .await?;

            for line in logs {
                all_logs.push(StackLogEntry {
                    container_name: container_name.clone(),
                    log: line,
                });
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
        let _ = self.get_stack(id, user_id).await?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        sqlx::query("UPDATE stacks SET compose_content = ?, updated_at = ? WHERE id = ?")
            .bind(compose_content)
            .bind(now)
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
pub struct StackHealth {
    pub status: String,
    pub total: usize,
    pub running: usize,
    pub stopped: usize,
    pub unhealthy: usize,
    pub containers: Vec<ContainerHealth>,
}

#[derive(serde::Serialize)]
pub struct ContainerHealth {
    pub id: String,
    pub name: String,
    pub state: String,
    pub status: String,
}

#[derive(serde::Serialize)]
pub struct StackLogEntry {
    pub container_name: String,
    pub log: String,
}
