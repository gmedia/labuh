use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogOutput, LogsOptions,
    RemoveContainerOptions, StartContainerOptions, StatsOptions, StopContainerOptions,
};
use bollard::image::{CreateImageOptions, ListImagesOptions, RemoveImageOptions};
use bollard::Docker;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
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
pub struct ContainerStats {
    pub cpu_percent: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub memory_percent: f64,
    pub network_rx: u64,
    pub network_tx: u64,
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

#[derive(Debug, Deserialize)]
pub struct CreateContainerRequest {
    pub name: String,
    pub image: String,
    pub env: Option<Vec<String>>,
    pub ports: Option<HashMap<String, String>>,
    pub volumes: Option<HashMap<String, String>>,
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
            })
            .collect())
    }

    /// Create a new container with full configuration
    pub async fn create_container(&self, request: CreateContainerRequest) -> Result<String> {
        use bollard::models::{HostConfig, PortBinding};
        use std::collections::HashMap;

        // Build exposed ports and port bindings
        let mut exposed_ports: HashMap<String, HashMap<(), ()>> = HashMap::new();
        let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();

        if let Some(ref ports) = request.ports {
            for (container_port, host_port) in ports {
                // Format: "80/tcp" or just "80"
                let port_key = if container_port.contains('/') {
                    container_port.clone()
                } else {
                    format!("{}/tcp", container_port)
                };

                // Add to exposed ports
                exposed_ports.insert(port_key.clone(), HashMap::new());

                // Add to port bindings
                port_bindings.insert(
                    port_key,
                    Some(vec![PortBinding {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some(host_port.clone()),
                    }]),
                );
            }
        }

        // Build volume bindings (host_path:container_path format)
        let binds: Option<Vec<String>> = request.volumes.map(|vols| {
            vols.into_iter()
                .map(|(host, container)| format!("{}:{}", host, container))
                .collect()
        });

        // Build host config
        let host_config = HostConfig {
            port_bindings: if port_bindings.is_empty() {
                None
            } else {
                Some(port_bindings)
            },
            binds,
            ..Default::default()
        };

        let config = Config {
            image: Some(request.image),
            env: request.env,
            exposed_ports: if exposed_ports.is_empty() {
                None
            } else {
                Some(exposed_ports)
            },
            host_config: Some(host_config),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: request.name,
            platform: None,
        };

        let response = self
            .docker
            .create_container(Some(options), config)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        Ok(response.id)
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

    /// Restart a container
    pub async fn restart_container(&self, id: &str) -> Result<()> {
        self.docker
            .restart_container(id, None)
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

    /// Get container logs
    pub async fn get_container_logs(&self, id: &str, tail: usize) -> Result<Vec<String>> {
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail.to_string(),
            ..Default::default()
        };

        let mut logs = self.docker.logs(id, Some(options));
        let mut result = Vec::new();

        while let Some(log) = logs.next().await {
            match log {
                Ok(output) => {
                    let line = match output {
                        LogOutput::StdOut { message } => {
                            String::from_utf8_lossy(&message).to_string()
                        }
                        LogOutput::StdErr { message } => {
                            String::from_utf8_lossy(&message).to_string()
                        }
                        LogOutput::Console { message } => {
                            String::from_utf8_lossy(&message).to_string()
                        }
                        LogOutput::StdIn { message } => {
                            String::from_utf8_lossy(&message).to_string()
                        }
                    };
                    result.push(line);
                }
                Err(e) => {
                    tracing::warn!("Error reading log: {}", e);
                }
            }
        }

        Ok(result)
    }

    /// Get container stats (one-shot)
    pub async fn get_container_stats(&self, id: &str) -> Result<ContainerStats> {
        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };

        let mut stats_stream = self.docker.stats(id, Some(options));

        if let Some(stats_result) = stats_stream.next().await {
            let stats = stats_result.map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

            // Calculate CPU percentage
            let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
                - stats.precpu_stats.cpu_usage.total_usage as f64;
            let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
                - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
            let cpu_percent = if system_delta > 0.0 && cpu_delta > 0.0 {
                let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;
                (cpu_delta / system_delta) * num_cpus * 100.0
            } else {
                0.0
            };

            // Memory stats
            let memory_usage = stats.memory_stats.usage.unwrap_or(0);
            let memory_limit = stats.memory_stats.limit.unwrap_or(1);
            let memory_percent = (memory_usage as f64 / memory_limit as f64) * 100.0;

            // Network stats
            let (network_rx, network_tx) = stats
                .networks
                .map(|nets| {
                    nets.values().fold((0u64, 0u64), |(rx, tx), net| {
                        (rx + net.rx_bytes, tx + net.tx_bytes)
                    })
                })
                .unwrap_or((0, 0));

            return Ok(ContainerStats {
                cpu_percent,
                memory_usage,
                memory_limit,
                memory_percent,
                network_rx,
                network_tx,
            });
        }

        Err(AppError::ContainerRuntime(
            "Failed to get stats".to_string(),
        ))
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
        let auth = credentials.map(|(username, password)| {
            bollard::auth::DockerCredentials {
                username: Some(username),
                password: Some(password),
                ..Default::default()
            }
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
