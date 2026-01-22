use std::sync::Arc;

use crate::error::{AppError, Result};
use crate::models::Project;
use crate::services::{CaddyService, ContainerService, ProjectService};

pub struct DeployService {
    container_service: Arc<ContainerService>,
    project_service: Arc<ProjectService>,
    caddy_service: Arc<CaddyService>,
    base_domain: String,
}

impl DeployService {
    pub fn new(
        container_service: Arc<ContainerService>,
        project_service: Arc<ProjectService>,
        caddy_service: Arc<CaddyService>,
        base_domain: String,
    ) -> Self {
        Self {
            container_service,
            project_service,
            caddy_service,
            base_domain,
        }
    }

    /// Deploy a project by creating/starting its container
    pub async fn deploy(&self, project_id: &str, user_id: &str) -> Result<Project> {
        // Get project
        let project = self
            .project_service
            .get_project(project_id, user_id)
            .await?;

        // Ensure project has an image
        let image = project.image.as_ref().ok_or_else(|| {
            AppError::Validation("Project must have an image configured".to_string())
        })?;

        // Stop and remove existing container if any
        if let Some(ref container_id) = project.container_id {
            let _ = self.container_service.stop_container(container_id).await;
            let _ = self
                .container_service
                .remove_container(container_id, true)
                .await;
        }

        // Pull the image first
        tracing::info!("Pulling image {} for project {}", image, project.name);
        self.container_service.pull_image(image).await?;

        // Parse environment variables
        let env_vars: Option<Vec<String>> = project.env_vars.as_ref().and_then(|v| {
            serde_json::from_str::<serde_json::Value>(v)
                .ok()
                .and_then(|json| {
                    if let serde_json::Value::Object(map) = json {
                        Some(
                            map.into_iter()
                                .map(|(k, v)| format!("{}={}", k, v.as_str().unwrap_or("")))
                                .collect(),
                        )
                    } else {
                        None
                    }
                })
        });

        // Create container name from slug
        let container_name = format!("labuh-{}", project.slug);

        // Create container
        let request = crate::services::container::CreateContainerRequest {
            name: container_name.clone(),
            image: image.clone(),
            env: env_vars,
            ports: None,
            volumes: None,
        };

        let container_id = self.container_service.create_container(request).await?;
        tracing::info!(
            "Created container {} for project {}",
            container_id,
            project.name
        );

        // Start container
        self.container_service
            .start_container(&container_id)
            .await?;
        tracing::info!(
            "Started container {} for project {}",
            container_id,
            project.name
        );

        // Update project with container ID and status
        self.project_service
            .update_project_status(project_id, "running", Some(&container_id))
            .await?;

        // Add Caddy route for the project
        if let Some(port) = project.port {
            let domain = format!("{}.{}", project.slug, self.base_domain);
            let upstream = format!("{}:{}", container_name, port);
            let _ = self.caddy_service.add_route(&domain, &upstream).await;
            tracing::info!("Added Caddy route: {} -> {}", domain, upstream);
        }

        // Return updated project
        self.project_service.get_project(project_id, user_id).await
    }

    /// Stop a deployed project
    pub async fn stop(&self, project_id: &str, user_id: &str) -> Result<Project> {
        let project = self
            .project_service
            .get_project(project_id, user_id)
            .await?;

        if let Some(ref container_id) = project.container_id {
            self.container_service.stop_container(container_id).await?;
            self.project_service
                .update_project_status(project_id, "stopped", Some(container_id))
                .await?;
        }

        // Remove Caddy route
        let domain = format!("{}.{}", project.slug, self.base_domain);
        let _ = self.caddy_service.remove_route(&domain).await;

        self.project_service.get_project(project_id, user_id).await
    }

    /// Restart a deployed project
    pub async fn restart(&self, project_id: &str, user_id: &str) -> Result<Project> {
        let project = self
            .project_service
            .get_project(project_id, user_id)
            .await?;

        if let Some(ref container_id) = project.container_id {
            self.container_service
                .restart_container(container_id)
                .await?;
            self.project_service
                .update_project_status(project_id, "running", Some(container_id))
                .await?;
        } else {
            return Err(AppError::Validation(
                "Project has no container to restart".to_string(),
            ));
        }

        self.project_service.get_project(project_id, user_id).await
    }
}
