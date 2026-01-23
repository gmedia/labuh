//! Stack service for managing Docker Compose stacks

use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;
use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::error::Result;
use crate::models::Stack;
use crate::services::compose::{parse_compose, service_to_container_request};
use crate::services::ContainerService;

pub struct StackService {
    db: SqlitePool,
    container_service: Arc<ContainerService>,
}

impl StackService {
    pub fn new(db: SqlitePool, container_service: Arc<ContainerService>) -> Self {
        Self { db, container_service }
    }

    /// List all stacks for a user
    pub async fn list_stacks(&self, user_id: &str) -> Result<Vec<Stack>> {
        let stacks = sqlx::query_as::<_, Stack>(
            "SELECT * FROM stacks WHERE user_id = ? ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(stacks)
    }

    /// Get a single stack
    pub async fn get_stack(&self, id: &str, user_id: &str) -> Result<Stack> {
        let stack = sqlx::query_as::<_, Stack>(
            "SELECT * FROM stacks WHERE id = ? AND user_id = ?"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Stack not found".to_string()))?;

        Ok(stack)
    }

    /// Create a new stack from docker-compose.yml
    pub async fn create_stack(&self, name: &str, compose_content: &str, user_id: &str) -> Result<Stack> {
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

        // Create containers for each service
        for service in &parsed.services {
            let request = service_to_container_request(service, &id, name);

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
    pub async fn get_stack_containers(&self, stack_id: &str) -> Result<Vec<crate::services::container::ContainerInfo>> {
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
                self.container_service.start_container(&container.id).await?;
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



// ... inside impl StackService

    /// Get a stack by ID (internal use)
    pub async fn get(&self, id: &str) -> Result<Stack> {
        let stack = sqlx::query_as::<_, Stack>(
            "SELECT * FROM stacks WHERE id = ?"
        )
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
            _ => Err(crate::error::AppError::Auth("Invalid webhook token".to_string())),
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

        let compose_content = stack.compose_content.ok_or_else(||
            crate::error::AppError::BadRequest("Stack has no compose content".to_string())
        )?;

        // Update status to deploying
        sqlx::query("UPDATE stacks SET status = ?, updated_at = ? WHERE id = ?")
            .bind("deploying")
            .bind(&Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.db)
            .await?;

        let parsed = parse_compose(&compose_content)?;

        // Recreate containers
        for service in &parsed.services {
            let request = service_to_container_request(service, &stack.id, &stack.name);

            // 1. Pull latest image
            self.container_service.pull_image(&request.image).await?;

            // 2. Stop and remove existing container if it exists
            // Finding the container ID depends on naming convention or tracking
            // Use get_stack_containers to find it
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

        // Update status to running (assuming start works, though create_container starts it?)
        // Note: create_container in ContainerService typically calculates config but doesn't auto-start?
        // Let's check ContainerService.create_container usage pattern.
        // In create_stack we pull/create, then set to stopped?
        // Wait, normally create_stack stops it? No, create_stack sets status directly.
        // Let's ensure we start them.

        self.start_stack(id, &stack.user_id).await?;

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
            self.container_service.remove_container(&container.id, true).await?;
        }

        // Delete stack from database
        sqlx::query("DELETE FROM stacks WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}
