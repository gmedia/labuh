//! Stack service for managing Docker Compose stacks

use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::Result;
use crate::models::Stack;
use crate::services::compose::{parse_compose, service_to_container_request};
use crate::services::{ContainerService, EnvironmentService};

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

    /// List all stacks for a user
    pub async fn list_stacks(&self, user_id: &str) -> Result<Vec<Stack>> {
        let stacks = sqlx::query_as::<_, Stack>(
            "SELECT * FROM stacks WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(stacks)
    }

    /// Get a single stack
    pub async fn get_stack(&self, id: &str, user_id: &str) -> Result<Stack> {
        let stack = sqlx::query_as::<_, Stack>("SELECT * FROM stacks WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Stack not found".to_string()))?;

        Ok(stack)
    }

    /// Create a new stack from docker-compose.yml
    pub async fn create_stack(
        &self,
        name: &str,
        compose_content: &str,
        user_id: &str,
    ) -> Result<Stack> {
        // Parse the compose file first
        let parsed = parse_compose(compose_content)?;

        // Create stack in database
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        sqlx::query(
            "INSERT INTO stacks (id, name, user_id, compose_content, status, webhook_token, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(name)
        .bind(user_id)
        .bind(compose_content)
        .bind("creating")
        .bind(&token)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        for service in &parsed.services {
            let mut request = service_to_container_request(service, &id, name);

            // Fetch merged env vars for this specific container
            let db_env = self
                .environment_service
                .get_env_map_for_container(&id, &service.name)
                .await
                .unwrap_or_default();

            if !db_env.is_empty() {
                let mut merged_env = request.env.unwrap_or_default();
                for (key, value) in &db_env {
                    let entry = format!("{}={}", key, value);
                    if let Some(pos) = merged_env
                        .iter()
                        .position(|e| e.starts_with(&format!("{}=", key)))
                    {
                        merged_env[pos] = entry;
                    } else {
                        merged_env.push(entry);
                    }
                }
                request.env = Some(merged_env);
            }

            // Pull image first
            self.container_service.pull_image(&request.image).await?;

            // Create container
            self.container_service.create_container(request).await?;
        }

        // Update stack status to stopped
        sqlx::query("UPDATE stacks SET status = ?, updated_at = ? WHERE id = ?")
            .bind("stopped")
            .bind(&Utc::now().to_rfc3339())
            .bind(&id)
            .execute(&self.db)
            .await?;

        self.get_stack(&id, user_id).await
    }

    /// Get containers belonging to a stack
    pub async fn get_stack_containers(
        &self,
        stack_id: &str,
    ) -> Result<Vec<crate::services::container::ContainerInfo>> {
        let all_containers = self.container_service.list_containers(true).await?;

        // Filter containers by name prefix (stack_name-service_name)
        let stack = sqlx::query_as::<_, Stack>("SELECT * FROM stacks WHERE id = ?")
            .bind(stack_id)
            .fetch_optional(&self.db)
            .await?;

        if let Some(stack) = stack {
            let prefix = format!("/{}-", stack.name);
            let containers: Vec<_> = all_containers
                .into_iter()
                .filter(|c| c.names.iter().any(|n| n.starts_with(&prefix)))
                .collect();
            return Ok(containers);
        }

        Ok(vec![])
    }

    /// Start all containers in a stack
    pub async fn start_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id).await?;

        for container in containers {
            if container.state != "running" {
                self.container_service
                    .start_container(&container.id)
                    .await?;
            }
        }

        sqlx::query("UPDATE stacks SET status = ?, updated_at = ? WHERE id = ?")
            .bind("running")
            .bind(&Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Stop all containers in a stack
    pub async fn stop_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id).await?;

        for container in containers {
            if container.state == "running" {
                self.container_service.stop_container(&container.id).await?;
            }
        }

        sqlx::query("UPDATE stacks SET status = ?, updated_at = ? WHERE id = ?")
            .bind("stopped")
            .bind(&Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Get a stack by ID (internal use)
    pub async fn get(&self, id: &str) -> Result<Stack> {
        let stack = sqlx::query_as::<_, Stack>("SELECT * FROM stacks WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Stack not found".to_string()))?;

        Ok(stack)
    }

    /// Validate webhook token and return stack
    pub async fn validate_webhook_token(&self, stack_id: &str, token: &str) -> Result<Stack> {
        let stack = self.get(stack_id).await?;

        match &stack.webhook_token {
            Some(t) if t == token => Ok(stack),
            _ => Err(crate::error::AppError::Auth(
                "Invalid webhook token".to_string(),
            )),
        }
    }

    /// Generate and save new webhook token
    pub async fn regenerate_webhook_token(&self, id: &str, user_id: &str) -> Result<String> {
        let _stack = self.get_stack(id, user_id).await?; // Verify ownership

        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        sqlx::query("UPDATE stacks SET webhook_token = ?, updated_at = ? WHERE id = ?")
            .bind(&token)
            .bind(&Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.db)
            .await?;

        Ok(token)
    }

    /// Redeploy an existing stack (pull images and recreate containers)
    pub async fn redeploy_stack(&self, id: &str) -> Result<()> {
        let stack = self.get(id).await?;

        let compose_content = stack.compose_content.ok_or_else(|| {
            crate::error::AppError::BadRequest("Stack has no compose content".to_string())
        })?;

        // Update status to deploying
        sqlx::query("UPDATE stacks SET status = ?, updated_at = ? WHERE id = ?")
            .bind("deploying")
            .bind(&Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.db)
            .await?;

        let parsed = parse_compose(&compose_content)?;

        for service in &parsed.services {
            let mut request = service_to_container_request(service, &stack.id, &stack.name);

            // Fetch merged env vars for this specific container
            let db_env = self
                .environment_service
                .get_env_map_for_container(id, &service.name)
                .await
                .unwrap_or_default();

            if !db_env.is_empty() {
                let mut merged_env = request.env.unwrap_or_default();
                for (key, value) in &db_env {
                    let entry = format!("{}={}", key, value);
                    if let Some(pos) = merged_env
                        .iter()
                        .position(|e| e.starts_with(&format!("{}=", key)))
                    {
                        merged_env[pos] = entry;
                    } else {
                        merged_env.push(entry);
                    }
                }
                request.env = Some(merged_env);
            }

            // 1. Pull latest image
            self.container_service.pull_image(&request.image).await?;

            // 2. Stop and remove existing container if it exists
            let containers = self.get_stack_containers(&stack.id).await?;
            let prefix = format!("/{}-{}", stack.name, service.name);

            for c in containers {
                if c.names.iter().any(|n| n == &prefix) {
                    let _ = self.container_service.stop_container(&c.id).await;
                    let _ = self.container_service.remove_container(&c.id, true).await;
                }
            }

            // 3. Create new container
            self.container_service.create_container(request).await?;
        }

        self.start_stack(id, &stack.user_id).await?;

        Ok(())
    }

    /// Redeploy only a specific service in a stack
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
        tracing::debug!(
            "Redeploying service '{}' in stack '{}'. Available services: {:?}",
            service_name,
            stack.name,
            parsed.services.iter().map(|s| &s.name).collect::<Vec<_>>()
        );

        let service = parsed
            .services
            .iter()
            .find(|s| {
                let normalized_name = service_name.to_lowercase();
                let s_name = s.name.to_lowercase();
                let full_name = format!("{}-{}", stack.name, s.name).to_lowercase();

                s_name == normalized_name || full_name == normalized_name
            })
            .ok_or_else(|| {
                tracing::error!(
                    "Service '{}' not found in stack '{}'. Available: {:?}",
                    service_name,
                    stack.name,
                    parsed.services.iter().map(|s| &s.name).collect::<Vec<_>>()
                );
                crate::error::AppError::NotFound(format!(
                    "Service {} not found in stack",
                    service_name
                ))
            })?;

        let mut request = service_to_container_request(service, &stack.id, &stack.name);

        // Fetch merged env vars for this specific container
        let db_env = self
            .environment_service
            .get_env_map_for_container(stack_id, service_name)
            .await?;

        if !db_env.is_empty() {
            let mut merged_env = request.env.unwrap_or_default();
            for (key, value) in &db_env {
                let entry = format!("{}={}", key, value);
                if let Some(pos) = merged_env
                    .iter()
                    .position(|e| e.starts_with(&format!("{}=", key)))
                {
                    merged_env[pos] = entry;
                } else {
                    merged_env.push(entry);
                }
            }
            request.env = Some(merged_env);
        }

        // 1. Pull latest image
        self.container_service.pull_image(&request.image).await?;

        // 2. Stop and remove existing container
        let containers = self.get_stack_containers(stack_id).await?;
        let full_name = format!("/{}-{}", stack.name, service_name);

        for c in containers {
            if c.names.iter().any(|n| n == &full_name) {
                let _ = self.container_service.stop_container(&c.id).await;
                let _ = self.container_service.remove_container(&c.id, true).await;
            }
        }

        // 3. Create and start new container
        let container_id = self.container_service.create_container(request).await?;
        self.container_service
            .start_container(&container_id)
            .await?;

        Ok(())
    }

    /// Remove a stack and all its containers
    pub async fn remove_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id).await?;

        // Stop and remove all containers
        for container in containers {
            if container.state == "running" {
                self.container_service.stop_container(&container.id).await?;
            }
            self.container_service
                .remove_container(&container.id, true)
                .await?;
        }

        // Delete stack from database
        sqlx::query("DELETE FROM stacks WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Get stack health overview
    pub async fn get_stack_health(&self, id: &str, user_id: &str) -> Result<StackHealth> {
        let _ = self.get_stack(id, user_id).await?; // Verify ownership
        let containers = self.get_stack_containers(id).await?;

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

    /// Get combined logs from all containers in a stack
    pub async fn get_stack_logs(
        &self,
        id: &str,
        user_id: &str,
        tail: Option<usize>,
    ) -> Result<Vec<StackLogEntry>> {
        let _ = self.get_stack(id, user_id).await?; // Verify ownership
        let containers = self.get_stack_containers(id).await?;

        let mut all_logs = Vec::new();
        let tail_count = tail.unwrap_or(100);

        for container in containers {
            let container_name = container
                .names
                .first()
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_else(|| container.id.clone());

            match self
                .container_service
                .get_container_logs(&container.id, tail_count)
                .await
            {
                Ok(logs) => {
                    for line in logs {
                        all_logs.push(StackLogEntry {
                            container: container_name.clone(),
                            message: line,
                        });
                    }
                }
                Err(e) => {
                    all_logs.push(StackLogEntry {
                        container: container_name.clone(),
                        message: format!("[error fetching logs: {}]", e),
                    });
                }
            }
        }

        Ok(all_logs)
    }

    /// Update stack compose content and redeploy
    pub async fn update_stack_compose(
        &self,
        id: &str,
        compose_content: &str,
        user_id: &str,
    ) -> Result<()> {
        let _stack = self.get_stack(id, user_id).await?; // Verify ownership

        // Validate compose first
        crate::services::compose::parse_compose(compose_content)?;

        sqlx::query("UPDATE stacks SET compose_content = ?, updated_at = ? WHERE id = ?")
            .bind(compose_content)
            .bind(&chrono::Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.db)
            .await?;

        // Trigger redeploy
        self.redeploy_stack(id).await?;

        Ok(())
    }
}

/// Stack health overview
#[derive(Debug, serde::Serialize)]
pub struct StackHealth {
    pub status: String,
    pub total: usize,
    pub running: usize,
    pub stopped: usize,
    pub unhealthy: usize,
    pub containers: Vec<ContainerHealth>,
}

#[derive(Debug, serde::Serialize)]
pub struct ContainerHealth {
    pub id: String,
    pub name: String,
    pub state: String,
    pub status: String,
}

/// Single log entry with container source
#[derive(Debug, serde::Serialize)]
pub struct StackLogEntry {
    pub container: String,
    pub message: String,
}
