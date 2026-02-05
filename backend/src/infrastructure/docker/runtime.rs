use async_trait::async_trait;
use bollard::Docker;
use bollard::container::LogOutput;
use bollard::models::{
    EndpointPortConfig, EndpointSettings, EndpointSpec, Limit, LocalNodeState,
    NetworkAttachmentConfig, NetworkConnectRequest, NetworkCreateRequest, ServiceSpec,
    ServiceSpecMode, ServiceSpecModeReplicated, SwarmInitRequest, SwarmJoinRequest, TaskSpec,
    TaskSpecContainerSpec, TaskSpecResources,
};
use bollard::query_parameters::{
    BuildImageOptions, CreateContainerOptions, CreateImageOptions, ListContainersOptions,
    ListImagesOptions, ListNetworksOptions, ListNodesOptions, LogsOptions, RemoveContainerOptions,
    RemoveImageOptions, StartContainerOptions, StatsOptions, StopContainerOptions,
    UpdateServiceOptions,
};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::runtime::{
    ContainerConfig, ContainerInfo, ContainerPort, EndpointInfo, NetworkInfo, RuntimePort,
    ServiceConfig, ServiceInfo,
};
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
            from_image: Some(image.to_string()),
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
        use bollard::models::{ContainerCreateBody, HostConfig, PortBinding};

        // Build exposed ports and port bindings
        let mut exposed_ports: Vec<String> = Vec::new();
        let mut port_bindings: bollard::models::PortMap = HashMap::new();

        if let Some(ports) = config.ports {
            for port_pair in ports {
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

                exposed_ports.push(port_key.clone());

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

        let bollard_config = ContainerCreateBody {
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
            name: Some(config.name),
            platform: "".to_string(),
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
            .start_container(id, None::<StartContainerOptions>)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;
        Ok(())
    }

    async fn stop_container(&self, id: &str) -> Result<()> {
        let options = StopContainerOptions {
            t: Some(10),
            signal: None,
        };
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
        let options = ListContainersOptions {
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
            .map(|c| {
                let networks = c
                    .network_settings
                    .and_then(|ns| ns.networks)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(name, settings)| {
                        (
                            name.clone(),
                            EndpointInfo {
                                name,
                                endpoint_id: settings.endpoint_id.unwrap_or_default(),
                                mac_address: settings.mac_address.unwrap_or_default(),
                                ipv4_address: settings.ip_address.unwrap_or_default(),
                                ipv6_address: settings.global_ipv6_address.unwrap_or_default(),
                            },
                        )
                    })
                    .collect();

                let ports = c.ports.map(|ps| {
                    ps.into_iter()
                        .map(|p| ContainerPort {
                            ip: p.ip,
                            private_port: p.private_port,
                            public_port: p.public_port,
                            port_type: p
                                .typ
                                .map(|t| t.to_string())
                                .unwrap_or_else(|| "tcp".to_string()),
                        })
                        .collect()
                });

                ContainerInfo {
                    id: c.id.unwrap_or_default(),
                    names: c.names.unwrap_or_default(),
                    image: c.image.unwrap_or_default(),
                    state: c.state.map(|s| s.to_string()).unwrap_or_default(),
                    status: c.status.unwrap_or_default(),
                    labels: c.labels.unwrap_or_default(),
                    networks,
                    ports,
                    created: c.created.unwrap_or(0),
                }
            })
            .collect())
    }

    async fn inspect_container(&self, id: &str) -> Result<ContainerInfo> {
        let container = self
            .docker
            .inspect_container(id, None)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        let networks = container
            .network_settings
            .as_ref()
            .and_then(|ns| ns.networks.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|(name, settings)| {
                (
                    name.clone(),
                    EndpointInfo {
                        name,
                        endpoint_id: settings.endpoint_id.unwrap_or_default(),
                        mac_address: settings.mac_address.unwrap_or_default(),
                        ipv4_address: settings.ip_address.unwrap_or_default(),
                        ipv6_address: settings.global_ipv6_address.unwrap_or_default(),
                    },
                )
            })
            .collect();

        let ports = container.network_settings.as_ref().and_then(|ns| {
            ns.ports.as_ref().map(|ps| {
                ps.iter()
                    .flat_map(|(k, v)| {
                        let parts: Vec<&str> = k.split('/').collect();
                        let private_port = parts[0].parse::<u16>().unwrap_or(0);
                        let port_type = parts.get(1).cloned().unwrap_or("tcp").to_string();

                        if let Some(bindings) = v {
                            bindings
                                .iter()
                                .map(|b| ContainerPort {
                                    ip: b.host_ip.clone(),
                                    private_port,
                                    public_port: b.host_port.as_ref().and_then(|p| p.parse().ok()),
                                    port_type: port_type.clone(),
                                })
                                .collect::<Vec<_>>()
                        } else {
                            vec![ContainerPort {
                                ip: None,
                                private_port,
                                public_port: None,
                                port_type,
                            }]
                        }
                    })
                    .collect()
            })
        });

        Ok(ContainerInfo {
            id: container.id.unwrap_or_default(),
            names: vec![container.name.unwrap_or_default()],
            image: container
                .config
                .as_ref()
                .and_then(|c| c.image.clone())
                .unwrap_or_else(|| "unknown".to_string()),
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
            networks,
            ports,
            created: container
                .created
                .and_then(|c| {
                    chrono::DateTime::parse_from_rfc3339(&c)
                        .ok()
                        .map(|dt| dt.timestamp())
                })
                .unwrap_or(0),
        })
    }

    async fn get_logs(&self, id: &str, tail: usize) -> Result<Vec<String>> {
        let options = LogsOptions {
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
            let stats = stats_result
                .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;

            // Calculate CPU percentage
            let cpu_usage = stats
                .cpu_stats
                .as_ref()
                .and_then(|c| c.cpu_usage.as_ref())
                .and_then(|u| u.total_usage)
                .unwrap_or(0);
            let precpu_usage = stats
                .precpu_stats
                .as_ref()
                .and_then(|c| c.cpu_usage.as_ref())
                .and_then(|u| u.total_usage)
                .unwrap_or(0);
            let cpu_delta = (cpu_usage as f64) - (precpu_usage as f64);

            let system_cpu = stats
                .cpu_stats
                .as_ref()
                .and_then(|c| c.system_cpu_usage)
                .unwrap_or(0);
            let presystem_cpu = stats
                .precpu_stats
                .as_ref()
                .and_then(|c| c.system_cpu_usage)
                .unwrap_or(0);
            let system_delta = (system_cpu as f64) - (presystem_cpu as f64);

            let cpu_percent = if system_delta > 0.0 && cpu_delta > 0.0 {
                let num_cpus = stats
                    .cpu_stats
                    .as_ref()
                    .and_then(|c| c.online_cpus)
                    .unwrap_or(1) as f64;
                (cpu_delta / system_delta) * num_cpus * 100.0
            } else {
                0.0
            };

            // Memory stats
            let memory_usage = stats
                .memory_stats
                .as_ref()
                .and_then(|m| m.usage)
                .unwrap_or(0);
            let memory_limit = stats
                .memory_stats
                .as_ref()
                .and_then(|m| m.limit)
                .unwrap_or(1);
            let memory_percent = (memory_usage as f64 / memory_limit as f64) * 100.0;

            // Network stats
            let (network_rx, network_tx) = stats
                .networks
                .map(
                    |nets: HashMap<String, bollard::models::ContainerNetworkStats>| {
                        nets.values().fold((0u64, 0u64), |(rx, tx), net| {
                            (
                                rx + net.rx_bytes.unwrap_or(0),
                                tx + net.tx_bytes.unwrap_or(0),
                            )
                        })
                    },
                )
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
        use tokio::sync::mpsc;

        let options = BuildImageOptions {
            t: Some(image_name.to_string()),
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
        let tar_data_stream = tar_data;

        tokio::spawn(async move {
            use http_body_util::Full;
            use hyper::body::Bytes;

            let body = Full::new(Bytes::from(tar_data_stream));
            let mut stream =
                docker.build_image(options, None, Some(http_body_util::Either::Left(body)));
            while let Some(res) = stream.next().await {
                match res {
                    Ok(inter) => {
                        if let Some(stream_msg) = inter.stream {
                            let _ = tx.send(Ok(stream_msg)).await;
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err::<String, AppError>(AppError::ContainerRuntime(
                                e.to_string(),
                            )))
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
        let options = ListImagesOptions {
            all: false,
            ..Default::default()
        };

        let images = self
            .docker
            .list_images(Some(options))
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;

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
        let options = RemoveImageOptions {
            force,
            ..Default::default()
        };

        self.docker
            .remove_image(id, Some(options), None)
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;

        Ok(())
    }

    async fn inspect_image(&self, id: &str) -> Result<crate::domain::runtime::ImageInspect> {
        let image = self
            .docker
            .inspect_image(id)
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;

        let config = image.config.unwrap_or_default();

        // Extract exposed ports - now a Vec<String> in 0.20
        let exposed_ports: Vec<String> = config.exposed_ports.unwrap_or_default();

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
        // Check if network exists
        if self.docker.inspect_network(name, None).await.is_ok() {
            return Ok(());
        }

        // If swarm is enabled, use overlay driver by default for new networks
        let is_swarm = self.is_swarm_enabled().await.unwrap_or(false);
        let driver = if is_swarm { "overlay" } else { "bridge" };

        let config = NetworkCreateRequest {
            name: name.to_string(),
            internal: Some(false),
            attachable: Some(true),
            driver: Some(driver.to_string()),
            ..Default::default()
        };

        if let Err(e) = self.docker.create_network(config).await {
            let err_str = e.to_string();
            if !err_str.contains("already exists") {
                return Err(AppError::ContainerRuntime(err_str));
            }
        }
        Ok(())
    }

    async fn connect_network(&self, container: &str, network: &str) -> Result<()> {
        let config = NetworkConnectRequest {
            container: container.to_string(),
            endpoint_config: Some(EndpointSettings::default()),
        };

        if let Err(e) = self.docker.connect_network(network, config).await {
            let err_str = e.to_string();
            // Ignore if already connected
            if !err_str.contains("already exists") && !err_str.contains("is already connected") {
                return Err(AppError::ContainerRuntime(err_str));
            }
        }
        Ok(())
    }

    async fn list_networks(&self) -> Result<Vec<NetworkInfo>> {
        let options = ListNetworksOptions::default();

        let networks = self
            .docker
            .list_networks(Some(options))
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        Ok(networks
            .into_iter()
            .map(|n| NetworkInfo {
                id: n.id.unwrap_or_default(),
                name: n.name.unwrap_or_default(),
                driver: n.driver.unwrap_or_default(),
                scope: n.scope.unwrap_or_default(),
                internal: n.internal.unwrap_or(false),
                attachable: n.attachable.unwrap_or(false),
                ingress: n.ingress.unwrap_or(false),
                labels: n.labels.unwrap_or_default(),
                containers: HashMap::new(), // Summary doesn't have containers, need to inspect if needed
            })
            .collect())
    }

    async fn migrate_network_to_overlay(&self, name: &str) -> Result<()> {
        // Check if network exists and its driver
        match self.docker.inspect_network(name, None).await {
            Ok(network) => {
                let driver = network.driver.unwrap_or_default();
                if driver == "overlay" {
                    tracing::info!("Network {} is already overlay, no migration needed", name);
                    return Ok(());
                }

                tracing::info!(
                    "Migrating network {} from {} to overlay for Swarm compatibility",
                    name,
                    driver
                );

                // Disconnect all containers from the network first
                if let Some(containers) = network.containers {
                    for (container_id, _) in containers {
                        tracing::debug!(
                            "Disconnecting container {} from network {}",
                            container_id,
                            name
                        );
                        let disconnect_config = bollard::models::NetworkDisconnectRequest {
                            container: container_id.clone(),
                            force: Some(true),
                        };
                        let _ = self
                            .docker
                            .disconnect_network(name, disconnect_config)
                            .await;
                    }
                }

                // Remove the old network
                if let Err(e) = self.docker.remove_network(name).await {
                    tracing::warn!("Failed to remove old network {}: {}", name, e);
                    // If we can't remove, try to proceed anyway
                }

                // Create new overlay network
                let config = NetworkCreateRequest {
                    name: name.to_string(),
                    internal: Some(false),
                    attachable: Some(true),
                    driver: Some("overlay".to_string()),
                    ..Default::default()
                };

                self.docker.create_network(config).await.map_err(|e| {
                    AppError::ContainerRuntime(format!(
                        "Failed to create overlay network {}: {}",
                        name, e
                    ))
                })?;

                tracing::info!("Successfully migrated network {} to overlay", name);
                Ok(())
            }
            Err(_) => {
                // Network doesn't exist, create as overlay
                tracing::info!("Network {} doesn't exist, creating as overlay", name);
                let config = NetworkCreateRequest {
                    name: name.to_string(),
                    internal: Some(false),
                    attachable: Some(true),
                    driver: Some("overlay".to_string()),
                    ..Default::default()
                };

                self.docker.create_network(config).await.map_err(|e| {
                    AppError::ContainerRuntime(format!(
                        "Failed to create overlay network {}: {}",
                        name, e
                    ))
                })?;
                Ok(())
            }
        }
    }

    async fn is_swarm_enabled(&self) -> Result<bool> {
        let info = self
            .docker
            .info()
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;
        Ok(info
            .swarm
            .and_then(|s| s.local_node_state)
            .map(|s| s == LocalNodeState::ACTIVE)
            .unwrap_or(false))
    }

    async fn swarm_init(&self, listen_addr: &str) -> Result<String> {
        let config = SwarmInitRequest {
            listen_addr: Some(listen_addr.to_string()),
            ..Default::default()
        };
        self.docker
            .init_swarm(config)
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;

        // After init, fetch the join token
        let swarm_info = self
            .docker
            .inspect_swarm()
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;
        swarm_info
            .join_tokens
            .and_then(|t| t.worker)
            .ok_or_else(|| AppError::ContainerRuntime("Failed to get swarm join token".to_string()))
    }

    async fn swarm_join(&self, listen_addr: &str, remote_addr: &str, token: &str) -> Result<()> {
        let config = SwarmJoinRequest {
            listen_addr: Some(listen_addr.to_string()),
            remote_addrs: Some(vec![remote_addr.to_string()]),
            join_token: Some(token.to_string()),
            ..Default::default()
        };
        self.docker
            .join_swarm(config)
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))
    }

    async fn list_nodes(&self) -> Result<Vec<crate::domain::runtime::SwarmNode>> {
        use crate::domain::runtime::{NodeResources, SwarmNode};
        let nodes = self
            .docker
            .list_nodes(None::<ListNodesOptions>)
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;
        Ok(nodes
            .into_iter()
            .map(|n| SwarmNode {
                id: n.id.unwrap_or_default(),
                hostname: n
                    .description
                    .as_ref()
                    .and_then(|d| d.hostname.clone())
                    .unwrap_or_default(),
                role: n
                    .spec
                    .as_ref()
                    .and_then(|s| s.role.as_ref().map(|r| r.to_string()))
                    .unwrap_or_default(),
                status: n
                    .status
                    .as_ref()
                    .and_then(|s| s.state.as_ref().map(|st| st.to_string()))
                    .unwrap_or_default(),
                availability: n
                    .spec
                    .as_ref()
                    .and_then(|s| s.availability.as_ref().map(|a| a.to_string()))
                    .unwrap_or_default(),
                addr: n
                    .status
                    .as_ref()
                    .and_then(|s| s.addr.clone())
                    .unwrap_or_default(),
                version: n
                    .description
                    .as_ref()
                    .and_then(|d| d.engine.as_ref().and_then(|e| e.engine_version.clone()))
                    .unwrap_or_default(),
                platform: format!(
                    "{}/{}",
                    n.description
                        .as_ref()
                        .and_then(|d| d.platform.as_ref().and_then(|p| p.os.clone()))
                        .unwrap_or_default(),
                    n.description
                        .as_ref()
                        .and_then(|d| d.platform.as_ref().and_then(|p| p.architecture.clone()))
                        .unwrap_or_default()
                ),
                resources: NodeResources {
                    nano_cpus: n
                        .description
                        .as_ref()
                        .and_then(|d| d.resources.as_ref().and_then(|r| r.nano_cpus))
                        .unwrap_or_default(),
                    memory_bytes: n
                        .description
                        .as_ref()
                        .and_then(|d| d.resources.as_ref().and_then(|r| r.memory_bytes))
                        .unwrap_or_default(),
                },
            })
            .collect())
    }

    async fn inspect_node(&self, id: &str) -> Result<crate::domain::runtime::SwarmNode> {
        use crate::domain::runtime::{NodeResources, SwarmNode};
        let n = self
            .docker
            .inspect_node(id)
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;
        Ok(SwarmNode {
            id: n.id.unwrap_or_default(),
            hostname: n
                .description
                .as_ref()
                .and_then(|d| d.hostname.clone())
                .unwrap_or_default(),
            role: n
                .spec
                .as_ref()
                .and_then(|s| s.role.as_ref().map(|r| r.to_string()))
                .unwrap_or_default(),
            status: n
                .status
                .as_ref()
                .and_then(|s| s.state.as_ref().map(|st| st.to_string()))
                .unwrap_or_default(),
            availability: n
                .spec
                .as_ref()
                .and_then(|s| s.availability.as_ref().map(|a| a.to_string()))
                .unwrap_or_default(),
            addr: n
                .status
                .as_ref()
                .and_then(|s| s.addr.clone())
                .unwrap_or_default(),
            version: n
                .description
                .as_ref()
                .and_then(|d| d.engine.as_ref().and_then(|e| e.engine_version.clone()))
                .unwrap_or_default(),
            platform: format!(
                "{}/{}",
                n.description
                    .as_ref()
                    .and_then(|d| d.platform.as_ref().and_then(|p| p.os.clone()))
                    .unwrap_or_default(),
                n.description
                    .as_ref()
                    .and_then(|d| d.platform.as_ref().and_then(|p| p.architecture.clone()))
                    .unwrap_or_default()
            ),
            resources: NodeResources {
                nano_cpus: n
                    .description
                    .as_ref()
                    .and_then(|d| d.resources.as_ref().and_then(|r| r.nano_cpus))
                    .unwrap_or_default(),
                memory_bytes: n
                    .description
                    .as_ref()
                    .and_then(|d| d.resources.as_ref().and_then(|r| r.memory_bytes))
                    .unwrap_or_default(),
            },
        })
    }

    async fn get_swarm_tokens(&self) -> Result<crate::domain::runtime::SwarmTokens> {
        use crate::domain::runtime::SwarmTokens;
        let info = self
            .docker
            .inspect_swarm()
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;

        let tokens = info.join_tokens.ok_or_else(|| {
            AppError::ContainerRuntime("Failed to get swarm join tokens".to_string())
        })?;

        Ok(SwarmTokens {
            manager: tokens.manager.unwrap_or_default(),
            worker: tokens.worker.unwrap_or_default(),
        })
    }

    async fn create_service(&self, config: ServiceConfig) -> Result<String> {
        let mut labels = config.labels.clone();
        labels.insert("labuh.managed".to_string(), "true".to_string());

        let container_spec = TaskSpecContainerSpec {
            image: Some(config.image),
            env: Some(config.env),
            labels: Some(labels.clone()),
            ..Default::default()
        };

        let resources = if config.cpu_limit.is_some() || config.memory_limit.is_some() {
            Some(TaskSpecResources {
                limits: Some(Limit {
                    nano_cpus: config.cpu_limit.map(|c| (c * 1e9) as i64),
                    memory_bytes: config.memory_limit,
                    ..Default::default()
                }),
                ..Default::default()
            })
        } else {
            None
        };

        let task_spec = TaskSpec {
            container_spec: Some(container_spec),
            resources,
            networks: Some(
                config
                    .networks
                    .iter()
                    .map(|n| NetworkAttachmentConfig {
                        target: Some(n.clone()),
                        ..Default::default()
                    })
                    .collect(),
            ),
            ..Default::default()
        };

        // Handle port mappings
        let endpoint_spec = if !config.ports.is_empty() {
            let ports = config
                .ports
                .iter()
                .filter_map(|p| {
                    let parts: Vec<&str> = p.split(':').collect();
                    if parts.len() == 2 {
                        let published = parts[0].parse::<i64>().ok();
                        let target = parts[1].parse::<i64>().ok();
                        Some(EndpointPortConfig {
                            protocol: Some(bollard::models::EndpointPortConfigProtocolEnum::TCP),
                            published_port: published,
                            target_port: target,
                            publish_mode: Some(
                                bollard::models::EndpointPortConfigPublishModeEnum::INGRESS,
                            ),
                            ..Default::default()
                        })
                    } else if parts.len() == 1 {
                        let target = parts[0].parse::<i64>().ok();
                        Some(EndpointPortConfig {
                            target_port: target,
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                })
                .collect();

            Some(EndpointSpec {
                mode: Some(bollard::models::EndpointSpecModeEnum::VIP),
                ports: Some(ports),
            })
        } else {
            None
        };

        let spec = ServiceSpec {
            name: Some(config.name),
            labels: Some(labels),
            task_template: Some(task_spec),
            mode: Some(ServiceSpecMode {
                replicated: Some(ServiceSpecModeReplicated {
                    replicas: Some(config.replicas as i64),
                }),
                ..Default::default()
            }),
            endpoint_spec,
            ..Default::default()
        };

        let response = self
            .docker
            .create_service(spec, None)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;

        Ok(response.id.unwrap_or_default())
    }

    async fn remove_service(&self, id_or_name: &str) -> Result<()> {
        self.docker
            .delete_service(id_or_name)
            .await
            .map_err(|e| AppError::ContainerRuntime(e.to_string()))?;
        Ok(())
    }

    async fn inspect_service(&self, name: &str) -> Result<Option<ServiceInfo>> {
        match self.docker.inspect_service(name, None).await {
            Ok(service) => {
                let version = service.version.and_then(|v| v.index).unwrap_or(0);

                let spec = service.spec.unwrap_or_default();
                let replicas = spec
                    .mode
                    .and_then(|m| m.replicated)
                    .and_then(|r| r.replicas)
                    .unwrap_or(1) as u64;

                let image = spec
                    .task_template
                    .and_then(|t| t.container_spec)
                    .and_then(|c| c.image)
                    .unwrap_or_default();

                Ok(Some(ServiceInfo {
                    id: service.id.unwrap_or_default(),
                    name: name.to_string(),
                    image,
                    replicas,
                    version,
                }))
            }
            Err(e) => {
                // If service not found, return None
                if e.to_string().contains("not found") || e.to_string().contains("no such service")
                {
                    Ok(None)
                } else {
                    Err(AppError::ContainerRuntime(e.to_string()))
                }
            }
        }
    }

    async fn update_service(&self, config: ServiceConfig) -> Result<()> {
        // 1. Inspect current service to get version
        let service = self
            .docker
            .inspect_service(&config.name, None)
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;

        let service_id = service
            .id
            .ok_or_else(|| AppError::Internal("Service ID missing".to_string()))?;
        let version = service
            .version
            .and_then(|v| v.index)
            .ok_or_else(|| AppError::Internal("Service version missing".to_string()))?;

        // 2. Build new spec (same logic as create_service)
        let mut labels = config.labels.clone();
        labels.insert("labuh.managed".to_string(), "true".to_string());

        let networks: Vec<NetworkAttachmentConfig> = config
            .networks
            .iter()
            .map(|n| NetworkAttachmentConfig {
                target: Some(n.clone()),
                ..Default::default()
            })
            .collect();

        let resources = if config.cpu_limit.is_some() || config.memory_limit.is_some() {
            Some(TaskSpecResources {
                limits: Some(Limit {
                    nano_cpus: config.cpu_limit.map(|c| (c * 1e9) as i64),
                    memory_bytes: config.memory_limit,
                    ..Default::default()
                }),
                ..Default::default()
            })
        } else {
            None
        };

        let placement = if !config.constraints.is_empty() {
            Some(bollard::models::TaskSpecPlacement {
                constraints: Some(config.constraints.clone()),
                ..Default::default()
            })
        } else {
            None
        };

        let task_spec = TaskSpec {
            container_spec: Some(TaskSpecContainerSpec {
                image: Some(config.image),
                env: if config.env.is_empty() {
                    None
                } else {
                    Some(config.env)
                },
                labels: Some(labels.clone()),
                ..Default::default()
            }),
            networks: if networks.is_empty() {
                None
            } else {
                Some(networks)
            },
            resources,
            placement,
            force_update: Some(1), // Force update for rolling deploy
            ..Default::default()
        };

        let endpoint_spec = if !config.ports.is_empty() {
            let ports: Vec<EndpointPortConfig> = config
                .ports
                .iter()
                .filter_map(|p| {
                    let parts: Vec<&str> = p.split(':').collect();
                    if parts.len() == 2 {
                        let published = parts[0].parse::<i64>().ok();
                        let target = parts[1].parse::<i64>().ok();
                        Some(EndpointPortConfig {
                            protocol: Some(bollard::models::EndpointPortConfigProtocolEnum::TCP),
                            published_port: published,
                            target_port: target,
                            publish_mode: Some(
                                bollard::models::EndpointPortConfigPublishModeEnum::INGRESS,
                            ),
                            ..Default::default()
                        })
                    } else if parts.len() == 1 {
                        let target = parts[0].parse::<i64>().ok();
                        Some(EndpointPortConfig {
                            target_port: target,
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                })
                .collect();

            Some(EndpointSpec {
                mode: Some(bollard::models::EndpointSpecModeEnum::VIP),
                ports: Some(ports),
            })
        } else {
            None
        };

        let spec = ServiceSpec {
            name: Some(config.name),
            labels: Some(labels),
            task_template: Some(task_spec),
            mode: Some(ServiceSpecMode {
                replicated: Some(ServiceSpecModeReplicated {
                    replicas: Some(config.replicas as i64),
                }),
                ..Default::default()
            }),
            endpoint_spec,
            ..Default::default()
        };

        // 3. Update the service
        let options = UpdateServiceOptions {
            version: version as i32,
            ..Default::default()
        };

        self.docker
            .update_service(&service_id, spec, options, None)
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;

        Ok(())
    }

    async fn update_service_scale(&self, service_name: &str, replicas: u64) -> Result<()> {
        use bollard::models::{ServiceSpecMode, ServiceSpecModeReplicated};

        // 1. Inspect service to get its current version and spec
        let service = self
            .docker
            .inspect_service(service_name, None)
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;

        let service_id = service
            .id
            .ok_or_else(|| AppError::Internal("Service ID missing".to_string()))?;
        let version = service
            .version
            .and_then(|v| v.index)
            .ok_or_else(|| AppError::Internal("Service version missing".to_string()))?;

        // 2. Prepare the new spec based on old one
        let mut spec = service.spec.unwrap_or_default();

        // Update replicas
        spec.mode = Some(ServiceSpecMode {
            replicated: Some(ServiceSpecModeReplicated {
                replicas: Some(replicas as i64),
            }),
            ..Default::default()
        });

        // 3. Update the service
        let options = UpdateServiceOptions {
            version: version as i32,
            ..Default::default()
        };

        self.docker
            .update_service(&service_id, spec, options, None)
            .await
            .map_err(|e: bollard::errors::Error| AppError::ContainerRuntime(e.to_string()))?;

        Ok(())
    }
}
