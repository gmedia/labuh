use bollard::container::{
    ListContainersOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::image::{CreateImageOptions, ListImagesOptions, RemoveImageOptions};
use bollard::Docker;
use futures::StreamExt;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize)]
pub struct ContainerInfo {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: Vec<PortMapping>,
    pub created: i64,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortMapping {
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub port_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageInfo {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub size: i64,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageInspect {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub exposed_ports: Vec<String>,
    pub env_vars: Vec<String>,
    pub working_dir: String,
    pub entrypoint: Vec<String>,
    pub cmd: Vec<String>,
    pub created: String,
    pub size: i64,
}

pub struct ContainerService {
    pub docker: Arc<Docker>,
}

impl ContainerService {
    pub async fn new() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        // Verify connection
        docker.ping().await.map_err(|e| {
            AppError::ContainerRuntime(format!("Failed to connect to Docker: {}", e))
        })?;

        Ok(Self {
            docker: Arc::new(docker),
        })
    }

    /// List all containers
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>> {
        let options = ListContainersOptions::<String> {
            all,
            ..Default::default()
        };

        let containers = self
            .docker
            .list_containers(Some(options))
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        Ok(containers
            .into_iter()
            .map(|c| ContainerInfo {
                id: c.id.unwrap_or_default(),
                names: c.names.unwrap_or_default(),
                image: c.image.unwrap_or_default(),
                state: c.state.unwrap_or_default(),
                status: c.status.unwrap_or_default(),
                ports: c
                    .ports
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| PortMapping {
                        private_port: p.private_port,
                        public_port: p.public_port,
                        port_type: p.typ.map(|t| t.to_string()).unwrap_or_default(),
                    })
                    .collect(),
                created: c.created.unwrap_or(0),
                labels: c.labels.unwrap_or_default(),
            })
            .collect())
    }

    /// Start a container
    pub async fn start_container(&self, id: &str) -> Result<()> {
        self.docker
            .start_container(id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;
        Ok(())
    }

    /// Stop a container
    pub async fn stop_container(&self, id: &str) -> Result<()> {
        let options = StopContainerOptions { t: 10 };
        self.docker
            .stop_container(id, Some(options))
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;
        Ok(())
    }

    /// Remove a container
    pub async fn remove_container(&self, id: &str, force: bool) -> Result<()> {
        let options = RemoveContainerOptions {
            force,
            ..Default::default()
        };
        self.docker
            .remove_container(id, Some(options))
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;
        Ok(())
    }

    /// List all images
    pub async fn list_images(&self) -> Result<Vec<ImageInfo>> {
        let options = ListImagesOptions::<String> {
            all: false,
            ..Default::default()
        };

        let images = self
            .docker
            .list_images(Some(options))
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        Ok(images
            .into_iter()
            .map(|i| ImageInfo {
                id: i.id,
                repo_tags: i.repo_tags,
                size: i.size,
                created: i.created,
            })
            .collect())
    }

    /// Pull an image from registry
    pub async fn pull_image(&self, image: &str) -> Result<()> {
        self.pull_image_with_auth(image, None).await
    }

    /// Pull an image with optional authentication
    pub async fn pull_image_with_auth(
        &self,
        image: &str,
        credentials: Option<(String, String)>,
    ) -> Result<()> {
        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };

        // Build auth config if credentials provided
        let auth = credentials.map(|(username, password)| bollard::auth::DockerCredentials {
            username: Some(username),
            password: Some(password),
            ..Default::default()
        });

        let mut stream = self.docker.create_image(Some(options), None, auth);

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    if let Some(status) = info.status {
                        tracing::debug!("Pull status: {}", status);
                    }
                }
                Err(e) => {
                    return Err(AppError::ContainerRuntime(format!(
                        "Failed to pull image: {}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Remove an image
    pub async fn remove_image(&self, id: &str, force: bool) -> Result<()> {
        let options = RemoveImageOptions {
            force,
            noprune: false,
        };

        self.docker
            .remove_image(id, Some(options), None)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        Ok(())
    }

    /// Inspect an image to get its config (exposed ports, env vars, etc.)
    pub async fn inspect_image(&self, id: &str) -> Result<ImageInspect> {
        let image = self
            .docker
            .inspect_image(id)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        let config = image.config.unwrap_or_default();

        // Extract exposed ports
        let exposed_ports: Vec<String> = config
            .exposed_ports
            .map(|ports| ports.keys().cloned().collect())
            .unwrap_or_default();

        // Extract environment variables
        let env_vars: Vec<String> = config.env.unwrap_or_default();

        // Extract working directory
        let working_dir = config.working_dir.unwrap_or_default();

        // Extract entrypoint and cmd
        let entrypoint = config.entrypoint.unwrap_or_default();
        let cmd = config.cmd.unwrap_or_default();

        Ok(ImageInspect {
            id: image.id.unwrap_or_default(),
            repo_tags: image.repo_tags.unwrap_or_default(),
            exposed_ports,
            env_vars,
            working_dir,
            entrypoint,
            cmd,
            created: image.created.unwrap_or_default(),
            size: image.size.unwrap_or(0),
        })
    }
}
