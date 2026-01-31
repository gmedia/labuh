use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::compose::{parse_compose, service_to_container_request};
use crate::domain::models::Stack;
use crate::domain::resource_repository::ResourceRepository;
use crate::domain::runtime::RuntimePort;
use crate::domain::stack_repository::StackRepository;
use crate::error::{AppError, Result};
use crate::usecase::environment::EnvironmentUsecase;
use crate::usecase::registry::RegistryUsecase;

use crate::domain::TeamRepository;

pub struct StackUsecase {
    repo: Arc<dyn StackRepository>,
    runtime: Arc<dyn RuntimePort>,
    environment_usecase: Arc<EnvironmentUsecase>,
    registry_usecase: Arc<RegistryUsecase>,
    resource_repo: Arc<dyn ResourceRepository>,
    team_repo: Arc<dyn TeamRepository>,
    git_service: Arc<crate::infrastructure::git::GitService>,
}

impl StackUsecase {
    pub fn new(
        repo: Arc<dyn StackRepository>,
        runtime: Arc<dyn RuntimePort>,
        environment_usecase: Arc<EnvironmentUsecase>,
        registry_usecase: Arc<RegistryUsecase>,
        resource_repo: Arc<dyn ResourceRepository>,
        team_repo: Arc<dyn TeamRepository>,
    ) -> Self {
        Self {
            repo,
            runtime,
            environment_usecase,
            registry_usecase,
            resource_repo,
            team_repo,
            git_service: Arc::new(crate::infrastructure::git::GitService::new()),
        }
    }

    pub fn runtime(&self) -> Arc<dyn RuntimePort> {
        self.runtime.clone()
    }

    pub async fn list_stacks(&self, user_id: &str) -> Result<Vec<Stack>> {
        let teams = self.team_repo.find_by_user_id(user_id).await?;
        let mut all_stacks = Vec::new();
        for team in teams {
            let mut stacks = self.repo.list_by_team(&team.id).await?;
            all_stacks.append(&mut stacks);
        }
        Ok(all_stacks)
    }

    pub async fn get_stack(&self, id: &str, user_id: &str) -> Result<Stack> {
        let stack = self.repo.find_by_id_internal(id).await?;
        let _role = self
            .team_repo
            .get_user_role(&stack.team_id, user_id)
            .await?
            .ok_or(AppError::Forbidden("Access denied".to_string()))?;
        Ok(stack)
    }

    pub async fn create_stack(
        &self,
        name: &str,
        compose_content: &str,
        user_id: &str,
        team_id: &str,
    ) -> Result<Stack> {
        let _role = self
            .team_repo
            .get_user_role(team_id, user_id)
            .await?
            .ok_or(AppError::Forbidden("Access denied".to_string()))?;

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
            team_id: team_id.to_string(),
            compose_content: Some(compose_content.to_string()),
            status: "creating".to_string(),
            webhook_token: Some(token),
            cron_schedule: None,
            health_check_path: None,
            health_check_interval: 30,
            last_stable_images: None,
            git_url: None,
            git_branch: None,
            last_commit_hash: None,
            created_at: now.clone(),
            updated_at: now,
        };

        self.repo.create(stack.clone()).await?;

        self.build_stack_services(&stack, compose_content).await?;

        self.repo.update_status(&id, "stopped").await?;
        self.get_stack(&id, user_id).await
    }

    pub async fn create_stack_from_git(
        &self,
        name: &str,
        git_url: &str,
        git_branch: &str,
        compose_path: &str,
        user_id: &str,
        team_id: &str,
    ) -> Result<Stack> {
        let _role = self
            .team_repo
            .get_user_role(team_id, user_id)
            .await?
            .ok_or(AppError::Forbidden("Access denied".to_string()))?;

        // 1. Setup git directory
        let id = Uuid::new_v4().to_string();
        let target_dir = format!("backend/data/git/{}", id);

        // 2. Clone repository
        let commit_hash = self
            .git_service
            .clone_or_pull(git_url, git_branch, &target_dir)
            .await?;

        // 3. Read compose content
        let full_compose_path = Path::new(&target_dir).join(compose_path);
        let compose_content = tokio::fs::read_to_string(full_compose_path)
            .await
            .map_err(|e| {
                AppError::BadRequest(format!("Failed to read compose file from repo: {}", e))
            })?;

        // 4. Create stack record
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
            team_id: team_id.to_string(),
            compose_content: Some(compose_content.clone()),
            status: "creating".to_string(),
            webhook_token: Some(token),
            cron_schedule: None,
            health_check_path: None,
            health_check_interval: 30,
            last_stable_images: None,
            git_url: Some(git_url.to_string()),
            git_branch: Some(git_branch.to_string()),
            last_commit_hash: Some(commit_hash),
            created_at: now.clone(),
            updated_at: now,
        };

        self.repo.create(stack.clone()).await?;

        // 5. Build services
        self.build_stack_services(&stack, &compose_content).await?;

        self.repo.update_status(&id, "stopped").await?;
        self.get_stack(&id, user_id).await
    }

    pub async fn sync_git(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let git_url = stack
            .git_url
            .clone()
            .ok_or_else(|| AppError::BadRequest("Stack not linked to Git".to_string()))?;
        let git_branch = stack
            .git_branch
            .clone()
            .unwrap_or_else(|| "main".to_string());

        let target_dir = format!("backend/data/git/{}", id);

        // 1. Pull latest
        let commit_hash = self
            .git_service
            .clone_or_pull(&git_url, &git_branch, &target_dir)
            .await?;

        // 2. Extract compose path (fixed to docker-compose.yml for now, or we could store it)
        let compose_path = "docker-compose.yml";
        let full_compose_path = Path::new(&target_dir).join(compose_path);
        let compose_content = tokio::fs::read_to_string(full_compose_path)
            .await
            .map_err(|e| {
                AppError::BadRequest(format!("Failed to read compose file from repo: {}", e))
            })?;

        // 3. Update stack record
        self.repo.update_compose(id, &compose_content).await?;
        self.repo.update_git_info(id, &commit_hash).await?;

        // 4. Redeploy
        self.redeploy_stack(id).await?;

        Ok(())
    }

    async fn build_stack_services(&self, stack: &Stack, compose_content: &str) -> Result<()> {
        let parsed = parse_compose(compose_content)?;

        for service in &parsed.services {
            let mut config = service_to_container_request(service, &stack.id, &stack.name);

            let db_env = self
                .environment_usecase
                .get_env_map_for_container(&stack.id, &service.name)
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

            let creds = self
                .registry_usecase
                .get_credentials_for_image_internal(&stack.team_id, &config.image)
                .await?;
            self.runtime.pull_image(&config.image, creds).await?;
            self.apply_resource_limits(&stack.id, &service.name, &mut config)
                .await?;
            self.runtime.create_container(config).await?;
        }
        Ok(())
    }

    pub async fn start_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id).await?;

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
        let containers = self.get_stack_containers(&stack.id).await?;

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

        // 1. Save current images for rollback
        self.save_stable_images(id).await?;

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

            let creds = self
                .registry_usecase
                .get_credentials_for_image_internal(&stack.team_id, &config.image)
                .await?;
            self.runtime.pull_image(&config.image, creds).await?;

            let containers = self.get_stack_containers(&stack.id).await?;
            let prefix = format!("/{}-{}", stack.name, service.name);
            for c in containers {
                if c.names.iter().any(|n| n == &prefix) {
                    let _ = self.runtime.stop_container(&c.id).await;
                    let _ = self.runtime.remove_container(&c.id, true).await;
                }
            }

            self.apply_resource_limits(id, &service.name, &mut config)
                .await?;
            self.runtime.create_container(config).await?;
        }

        self.start_stack(id, &stack.user_id).await?;

        // 2. Perform health check
        if let Err(e) = self.perform_health_check(id).await {
            tracing::error!(
                "Health check failed for stack {}: {}. Triggering rollback...",
                id,
                e
            );
            self.rollback_stack(id, &stack.user_id).await?;
            return Err(e);
        }

        Ok(())
    }

    pub async fn remove_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let containers = self.get_stack_containers(&stack.id).await?;

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
        let containers = self.get_stack_containers(&stack.id).await?;

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
        let containers = self.get_stack_containers(&stack.id).await?;

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

    pub async fn update_automation(
        &self,
        id: &str,
        user_id: &str,
        cron: Option<String>,
        health_path: Option<String>,
        health_interval: i32,
    ) -> Result<()> {
        let _stack = self.get_stack(id, user_id).await?;
        self.repo
            .update_automation(id, cron, health_path, health_interval)
            .await?;
        Ok(())
    }

    pub async fn perform_health_check(&self, id: &str) -> Result<()> {
        let stack = self.repo.find_by_id_internal(id).await?;
        let health_path = match &stack.health_check_path {
            Some(path) if !path.is_empty() => path,
            _ => return Ok(()), // No health check configured
        };

        tracing::info!(
            "Performing health check for stack {} on {}",
            id,
            health_path
        );

        // Wait for containers to settle
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        if health_path.starts_with("http") {
            let client = reqwest::Client::builder()
                .timeout(tokio::time::Duration::from_secs(10))
                .build()
                .map_err(|e| AppError::Internal(e.to_string()))?;

            let res =
                client.get(health_path).send().await.map_err(|e| {
                    AppError::Internal(format!("Health check request failed: {}", e))
                })?;

            if !res.status().is_success() {
                return Err(AppError::Internal(format!(
                    "Health check returned non-success status: {}",
                    res.status()
                )));
            }
        }

        // For now we only support HTTP health checks.
        // Command execution in containers would require adding `exec` to RuntimePort.

        Ok(())
    }

    pub async fn save_stable_images(&self, id: &str) -> Result<()> {
        let stack = self.repo.find_by_id_internal(id).await?;
        let containers = self.get_stack_containers(id).await?;

        let mut images = std::collections::HashMap::new();
        for container in containers {
            // Find the service name from labels
            if let Some(service_name) = container.labels.get("com.docker.compose.service") {
                images.insert(service_name.clone(), container.image.clone());
            } else {
                // Fallback to searching names
                let prefix = format!("/{}-", stack.name);
                for name in &container.names {
                    if name.starts_with(&prefix) {
                        let service_name = name.replacen(&prefix, "", 1);
                        images.insert(service_name, container.image.clone());
                    }
                }
            }
        }

        if !images.is_empty() {
            let json =
                serde_json::to_string(&images).map_err(|e| AppError::Internal(e.to_string()))?;
            self.repo.update_last_stable_images(id, Some(json)).await?;
        }

        Ok(())
    }

    pub async fn rollback_stack(&self, id: &str, user_id: &str) -> Result<()> {
        let stack = self.get_stack(id, user_id).await?;
        let stable_images_json = stack.last_stable_images.ok_or_else(|| {
            AppError::BadRequest("No stable version available for rollback".to_string())
        })?;

        let stable_images: std::collections::HashMap<String, String> =
            serde_json::from_str(&stable_images_json)
                .map_err(|e| AppError::Internal(e.to_string()))?;

        let compose_content = stack
            .compose_content
            .ok_or_else(|| AppError::BadRequest("Stack has no compose content".to_string()))?;

        self.repo.update_status(id, "rolling_back").await?;
        let parsed = parse_compose(&compose_content)?;

        for service in &parsed.services {
            // Check if we have a stable image for this service
            let image = match stable_images.get(&service.name) {
                Some(img) => img,
                None => continue, // Skip services that aren't in the stable list
            };

            let mut config = service_to_container_request(service, &stack.id, &stack.name);
            config.image = image.clone();

            // Apply env vars
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

            // Pull stable image anyway to be sure
            let creds = self
                .registry_usecase
                .get_credentials_for_image_internal(&stack.team_id, &config.image)
                .await?;
            self.runtime.pull_image(&config.image, creds).await?;

            let containers = self.get_stack_containers(&stack.id).await?;
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
        self.repo.update_status(id, "rolled_back").await?;
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

        let creds = self
            .registry_usecase
            .get_credentials_for_image_internal(&stack.team_id, &config.image)
            .await?;
        self.runtime.pull_image(&config.image, creds).await?;
        let containers = self.get_stack_containers(&stack.id).await?;
        let prefix = format!("/{}-{}", stack.name, service.name);
        for c in containers {
            if c.names.iter().any(|n| n == &prefix) {
                let _ = self.runtime.stop_container(&c.id).await;
                let _ = self.runtime.remove_container(&c.id, true).await;
            }
        }
        self.apply_resource_limits(stack_id, &service.name, &mut config)
            .await?;
        self.runtime.create_container(config).await?;
        Ok(())
    }

    pub async fn validate_webhook_token(&self, id: &str, token: &str) -> Result<Stack> {
        self.repo.validate_webhook_token(id, token).await
    }

    pub async fn get_stack_containers(
        &self,
        stack_id: &str,
    ) -> Result<Vec<crate::domain::runtime::ContainerInfo>> {
        let all = self.runtime.list_containers(true).await?;
        Ok(all
            .into_iter()
            .filter(|c| {
                c.labels
                    .get("labuh.stack.id")
                    .map(|id| id == stack_id)
                    .unwrap_or(false)
            })
            .collect())
    }

    pub async fn verify_container_ownership(
        &self,
        container_id: &str,
        user_id: &str,
    ) -> Result<crate::domain::runtime::ContainerInfo> {
        let container = self.runtime.inspect_container(container_id).await?;
        let stack_id = container
            .labels
            .get("labuh.stack.id")
            .ok_or_else(|| AppError::Forbidden("Container not managed by Labuh".to_string()))?;

        // Verify stack ownership
        self.repo.find_by_id(stack_id, user_id).await?;

        Ok(container)
    }

    pub async fn start_container(&self, container_id: &str, user_id: &str) -> Result<()> {
        self.verify_container_ownership(container_id, user_id)
            .await?;
        self.runtime.start_container(container_id).await
    }

    pub async fn stop_container(&self, container_id: &str, user_id: &str) -> Result<()> {
        self.verify_container_ownership(container_id, user_id)
            .await?;
        self.runtime.stop_container(container_id).await
    }

    pub async fn restart_container(&self, container_id: &str, user_id: &str) -> Result<()> {
        self.verify_container_ownership(container_id, user_id)
            .await?;
        self.runtime.restart_container(container_id).await
    }

    pub async fn remove_container(&self, container_id: &str, user_id: &str) -> Result<()> {
        self.verify_container_ownership(container_id, user_id)
            .await?;
        self.runtime.remove_container(container_id, true).await
    }

    pub async fn get_container_logs(
        &self,
        container_id: &str,
        user_id: &str,
        tail: usize,
    ) -> Result<Vec<String>> {
        self.verify_container_ownership(container_id, user_id)
            .await?;
        self.runtime.get_logs(container_id, tail).await
    }

    pub async fn get_container_stats(
        &self,
        container_id: &str,
        user_id: &str,
    ) -> Result<crate::domain::runtime::ContainerStats> {
        self.verify_container_ownership(container_id, user_id)
            .await?;
        self.runtime.get_stats(container_id).await
    }

    pub async fn get_stack_backup(
        &self,
        id: &str,
        user_id: &str,
    ) -> Result<crate::domain::models::stack::StackBackup> {
        let stack = self.get_stack(id, user_id).await?;
        let env_vars = self.environment_usecase.get_raw_vars(&stack.id).await?;

        let backup_envs = env_vars
            .into_iter()
            .map(|e| crate::domain::models::stack::BackupEnvVar {
                container_name: e.container_name,
                key: e.key,
                value: e.value,
                is_secret: e.is_secret,
            })
            .collect();

        Ok(crate::domain::models::stack::StackBackup {
            name: stack.name,
            compose_content: stack.compose_content.unwrap_or_default(),
            env_vars: backup_envs,
        })
    }

    pub async fn restore_stack(
        &self,
        backup: crate::domain::models::stack::StackBackup,
        user_id: &str,
        team_id: &str,
    ) -> Result<Stack> {
        // 1. Create the stack
        let stack = self
            .create_stack(&backup.name, &backup.compose_content, user_id, team_id)
            .await?;

        // 2. Restore env vars
        for env in backup.env_vars {
            self.environment_usecase
                .set_var(
                    &stack.id,
                    &env.container_name,
                    &env.key,
                    &env.value,
                    env.is_secret,
                )
                .await?;
        }

        // 3. Redeploy to apply the newly set env vars
        self.redeploy_stack(&stack.id).await?;

        self.get_stack(&stack.id, user_id).await
    }

    async fn apply_resource_limits(
        &self,
        stack_id: &str,
        service_name: &str,
        config: &mut crate::domain::runtime::ContainerConfig,
    ) -> Result<()> {
        if let Some(limits) = self
            .resource_repo
            .get_resource_limits(stack_id, service_name)
            .await?
        {
            config.cpu_limit = limits.cpu_limit;
            config.memory_limit = limits.memory_limit;
        }
        Ok(())
    }
}
