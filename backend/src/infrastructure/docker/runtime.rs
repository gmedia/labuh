use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogOutput, LogsOptions,
    RemoveContainerOptions, StartContainerOptions, StatsOptions, StopContainerOptions,
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
    async fn pull_image(&self, image: &str, credentials: Option<(String, String)>) -> Result<()> {
        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };

        let auth = credentials.map(|(u, p)| bollard::auth::DockerCredentials {
            username: Some(u),
            password: Some(p),
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

    async fn create_container(&self, config: ContainerConfig) -> Result<String> {
        use bollard::models::{HostConfig, PortBinding};

        // Build exposed ports and port bindings
        let mut exposed_ports: HashMap<String, HashMap<(), ()>> = HashMap::new();
        let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();

        if let Some(ports) = config.ports {
            for port_pair in ports {
                // Supports:
                // - "80" -> Container 80
                // - "80:80" -> Host 80:Container 80 (0.0.0.0)
                // - "127.0.0.1:80:80" -> Host 80:Container 80 (127.0.0.1)

                let parts: Vec<&str> = port_pair.split(':').collect();
                let (host_ip, host_port, container_port) = match parts.len() {
                    3 => (
                        Some(parts[0].to_string()),
                        Some(parts[1].to_string()),
                        parts[2].to_string(),
                    ),
                    2 => (
                        Some("0.0.0.0".to_string()),
                        Some(parts[0].to_string()),
                        parts[1].to_string(),
                    ),
                    _ => (None, None, parts[0].to_string()),
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
                            host_ip: host_ip.or(Some("0.0.0.0".to_string())),
                            host_port: Some(hp),
                        }]),
                    );
                }
            }
        }

        // Build volume bindings
        let binds: Option<Vec<String>> = config.volumes;

        // Build restart policy
        let restart_policy = config.restart_policy.map(|p| {
            use bollard::models::{RestartPolicy, RestartPolicyNameEnum};
            let name = match p.to_lowercase().as_str() {
                "always" => RestartPolicyNameEnum::ALWAYS,
                "unless-stopped" => RestartPolicyNameEnum::UNLESS_STOPPED,
                "on-failure" => RestartPolicyNameEnum::ON_FAILURE,
                "no" => RestartPolicyNameEnum::NO,
                _ => RestartPolicyNameEnum::NO,
            };
            RestartPolicy {
                name: Some(name),
                maximum_retry_count: None,
            }
        });

        let host_config = HostConfig {
            port_bindings: if port_bindings.is_empty() {
                None
            } else {
                Some(port_bindings)
            },
            binds,
            memory: config.memory_limit,
            nano_cpus: config.cpu_limit.map(|c| (c * 1e9) as i64),
            network_mode: config.network_mode,
            extra_hosts: config.extra_hosts,
            restart_policy,
            ..Default::default()
        };

        let bollard_config = Config {
            image: Some(config.image),
            env: config.env,
            cmd: config.cmd,
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

    async fn restart_container(&self, id: &str) -> Result<()> {
        self.docker
            .restart_container(id, None)
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
                labels: c.labels.unwrap_or_default(),
            })
            .collect())
    }

    async fn inspect_container(&self, id: &str) -> Result<ContainerInfo> {
        let container = self
            .docker
            .inspect_container(id, None)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        Ok(ContainerInfo {
            id: container.id.unwrap_or_default(),
            names: container.name.map(|n| vec![n]).unwrap_or_default(),
            image: container
                .config
                .as_ref()
                .and_then(|c| c.image.clone())
                .unwrap_or_default(),
            state: container
                .state
                .as_ref()
                .and_then(|s| s.status.as_ref())
                .map(|s| s.to_string())
                .unwrap_or_default(),
            status: container
                .state
                .as_ref()
                .and_then(|s| s.status.as_ref())
                .map(|s| s.to_string())
                .unwrap_or_default(),
            labels: container
                .config
                .as_ref()
                .and_then(|c| c.labels.clone())
                .unwrap_or_default(),
        })
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

    async fn get_stats(&self, id: &str) -> Result<crate::domain::runtime::ContainerStats> {
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

            return Ok(crate::domain::runtime::ContainerStats {
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

    async fn build_image(
        &self,
        image_name: &str,
        context_path: &str,
        dockerfile_path: &str,
    ) -> Result<tokio_stream::wrappers::ReceiverStream<Result<String>>> {
        use bollard::image::BuildImageOptions;
        use tokio::sync::mpsc;

        let options = BuildImageOptions {
            t: image_name.to_string(),
            dockerfile: dockerfile_path.to_string(),
            rm: true,
            ..Default::default()
        };

        // Create tarball from context_path
        let mut tar = tar::Builder::new(Vec::new());
        tar.append_dir_all(".", context_path).map_err(|e| {
            AppError::Internal(format!("Failed to create build context tar: {}", e))
        })?;
        let tar_data = tar.into_inner().map_err(|e| {
            AppError::Internal(format!("Failed to finalize build context tar: {}", e))
        })?;

        let (tx, rx) = mpsc::channel(100);
        let docker = self.docker.clone();
        let tar_data_stream = tar_data.clone();

        tokio::spawn(async move {
            let mut stream = docker.build_image(options, None, Some(tar_data_stream.into()));
            while let Some(res) = stream.next().await {
                match res {
                    Ok(inter) => {
                        if let Some(stream_msg) = inter.stream {
                            let _ = tx.send(Ok(stream_msg)).await;
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(AppError::ContainerRuntime(e.to_string())))
                            .await;
                        break;
                    }
                }
            }
        });

        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    async fn exec_command(
        &self,
        id: &str,
        cmd: Vec<String>,
    ) -> Result<bollard::exec::CreateExecResults> {
        use bollard::exec::CreateExecOptions;

        let options = CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            attach_stdin: Some(true),
            tty: Some(true),
            cmd: Some(cmd),
            ..Default::default()
        };

        let exec = self
            .docker
            .create_exec(id, options)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        Ok(exec)
    }

    async fn connect_exec(&self, exec_id: &str) -> Result<bollard::exec::StartExecResults> {
        use bollard::exec::StartExecOptions;

        let options = StartExecOptions {
            detach: false,
            tty: true,
            ..Default::default()
        };

        self.docker
            .start_exec(exec_id, Some(options))
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))
    }

    async fn list_images(&self) -> Result<Vec<crate::domain::runtime::ImageInfo>> {
        use bollard::image::ListImagesOptions;
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
            .map(|i| crate::domain::runtime::ImageInfo {
                id: i.id,
                repo_tags: i.repo_tags,
                size: i.size,
                created: i.created,
            })
            .collect())
    }

    async fn remove_image(&self, id: &str, force: bool) -> Result<()> {
        use bollard::image::RemoveImageOptions;
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

    async fn inspect_image(&self, id: &str) -> Result<crate::domain::runtime::ImageInspect> {
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

        Ok(crate::domain::runtime::ImageInspect {
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

    async fn ensure_network(&self, name: &str) -> Result<()> {
        use bollard::network::CreateNetworkOptions;

        // Check if exists
        if self
            .docker
            .inspect_network::<String>(name, None)
            .await
            .is_ok()
        {
            return Ok(());
        }

        let options = CreateNetworkOptions {
            name,
            driver: "bridge",
            ..Default::default()
        };

        self.docker
            .create_network(options)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;
        Ok(())
    }

    async fn connect_network(&self, container: &str, network: &str) -> Result<()> {
        use bollard::models::EndpointSettings;
        use bollard::network::ConnectNetworkOptions;

        let config = ConnectNetworkOptions {
            container,
            endpoint_config: EndpointSettings::default(),
        };

        match self.docker.connect_network(network, config).await {
            Ok(_) => Ok(()),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("403") || err_str.contains("already exists") {
                    Ok(())
                } else {
                    Err(crate::error::AppError::ContainerRuntime(err_str))
                }
            }
        }
    }
}
