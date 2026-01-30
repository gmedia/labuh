use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogOutput, LogsOptions,
    RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::runtime::{ContainerConfig, ContainerInfo, RuntimePort};
use crate::error::{AppError, Result};

pub struct DockerRuntimeAdapter {
    docker: Arc<Docker>,
}

impl DockerRuntimeAdapter {
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
}

#[async_trait]
impl RuntimePort for DockerRuntimeAdapter {
    async fn pull_image(&self, image: &str) -> Result<()> {
        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };

        let mut stream = self.docker.create_image(Some(options), None, None);

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

    async fn create_container(&self, config: ContainerConfig) -> Result<String> {
        use bollard::models::{HostConfig, PortBinding};

        // Build exposed ports and port bindings
        let mut exposed_ports: HashMap<String, HashMap<(), ()>> = HashMap::new();
        let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();

        if let Some(ports) = config.ports {
            for port_pair in ports {
                // simple "host:container" or "container"
                let parts: Vec<&str> = port_pair.split(':').collect();
                let (host_port, container_port) = if parts.len() == 2 {
                    (Some(parts[0].to_string()), parts[1].to_string())
                } else {
                    (None, parts[0].to_string())
                };

                let port_key = if container_port.contains('/') {
                    container_port.clone()
                } else {
                    format!("{}/tcp", container_port)
                };

                exposed_ports.insert(port_key.clone(), HashMap::new());

                if let Some(hp) = host_port {
                    port_bindings.insert(
                        port_key,
                        Some(vec![PortBinding {
                            host_ip: Some("0.0.0.0".to_string()),
                            host_port: Some(hp),
                        }]),
                    );
                }
            }
        }

        // Build volume bindings
        let binds: Option<Vec<String>> = config.volumes;

        let host_config = HostConfig {
            port_bindings: if port_bindings.is_empty() {
                None
            } else {
                Some(port_bindings)
            },
            binds,
            ..Default::default()
        };

        let bollard_config = Config {
            image: Some(config.image),
            env: config.env,
            exposed_ports: if exposed_ports.is_empty() {
                None
            } else {
                Some(exposed_ports)
            },
            host_config: Some(host_config),
            labels: config.labels,
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: config.name,
            platform: None,
        };

        let response = self
            .docker
            .create_container(Some(options), bollard_config)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        Ok(response.id)
    }

    async fn start_container(&self, id: &str) -> Result<()> {
        self.docker
            .start_container(id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;
        Ok(())
    }

    async fn stop_container(&self, id: &str) -> Result<()> {
        let options = StopContainerOptions { t: 10 };
        self.docker
            .stop_container(id, Some(options))
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;
        Ok(())
    }

    async fn remove_container(&self, id: &str, force: bool) -> Result<()> {
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

    async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>> {
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
            })
            .collect())
    }

    async fn get_logs(&self, id: &str, tail: usize) -> Result<Vec<String>> {
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
}
