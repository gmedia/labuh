//! Network service for managing Docker networks
//!
//! Handles labuh-network creation and container connections.

use bollard::models::EndpointSettings;
use bollard::network::{ConnectNetworkOptions, CreateNetworkOptions};
use std::sync::Arc;

use crate::error::{AppError, Result};
use crate::services::ContainerService;

/// Network name used for all Labuh containers
pub const LABUH_NETWORK: &str = "labuh-network";

pub struct NetworkService {
    container_service: Arc<ContainerService>,
}

impl NetworkService {
    pub fn new(container_service: Arc<ContainerService>) -> Self {
        Self { container_service }
    }

    /// Ensure labuh-network exists, create if not
    pub async fn ensure_labuh_network(&self) -> Result<()> {
        let docker = &self.container_service.docker;

        // Check if network exists
        match docker.inspect_network::<String>(LABUH_NETWORK, None).await {
            Ok(_) => {
                tracing::debug!("Network {} already exists", LABUH_NETWORK);
                return Ok(());
            }
            Err(_) => {
                tracing::info!("Creating network: {}", LABUH_NETWORK);
            }
        }

        // Create the network
        let options = CreateNetworkOptions {
            name: LABUH_NETWORK,
            driver: "bridge",
            ..Default::default()
        };

        docker
            .create_network(options)
            .await
            .map_err(|e| AppError::ContainerRuntime(format!("Failed to create network: {}", e)))?;

        tracing::info!("Created network: {}", LABUH_NETWORK);
        Ok(())
    }

    /// Connect a container to labuh-network
    pub async fn connect_container(&self, container_id: &str) -> Result<()> {
        let docker = &self.container_service.docker;

        let config = ConnectNetworkOptions {
            container: container_id,
            endpoint_config: EndpointSettings::default(),
        };

        match docker.connect_network(LABUH_NETWORK, config).await {
            Ok(_) => {
                tracing::debug!("Connected container {} to {}", container_id, LABUH_NETWORK);
                Ok(())
            }
            Err(e) => {
                // Ignore if already connected (status 403 Conflict)
                if e.to_string().contains("403") || e.to_string().contains("already exists") {
                    tracing::debug!(
                        "Container {} already connected to {}",
                        container_id,
                        LABUH_NETWORK
                    );
                    Ok(())
                } else {
                    Err(AppError::ContainerRuntime(format!(
                        "Failed to connect container {} to {}: {}",
                        container_id, LABUH_NETWORK, e
                    )))
                }
            }
        }
    }

    /// Connect Caddy to a specific network (for stack isolation)
    #[allow(dead_code)]
    pub async fn connect_caddy_to_network(&self, network: &str) -> Result<()> {
        let docker = &self.container_service.docker;

        let config = ConnectNetworkOptions {
            container: "labuh-caddy",
            endpoint_config: EndpointSettings::default(),
        };

        // Ignore errors if already connected
        let _ = docker.connect_network(network, config).await;

        tracing::debug!("Connected labuh-caddy to network: {}", network);
        Ok(())
    }

    /// Disconnect a container from labuh-network
    #[allow(dead_code)]
    pub async fn disconnect_container(&self, container_id: &str) -> Result<()> {
        let docker = &self.container_service.docker;

        docker
            .disconnect_network(
                LABUH_NETWORK,
                bollard::network::DisconnectNetworkOptions {
                    container: container_id,
                    force: false,
                },
            )
            .await
            .map_err(|e| {
                AppError::ContainerRuntime(format!("Failed to disconnect container: {}", e))
            })?;

        Ok(())
    }

    /// List all containers connected to labuh-network
    #[allow(dead_code)]
    pub async fn list_connected_containers(&self) -> Result<Vec<String>> {
        let docker = &self.container_service.docker;

        let network = docker
            .inspect_network::<String>(LABUH_NETWORK, None)
            .await
            .map_err(|e| AppError::ContainerRuntime(format!("Failed to inspect network: {}", e)))?;

        let containers = network
            .containers
            .unwrap_or_default()
            .keys()
            .cloned()
            .collect();

        Ok(containers)
    }

    /// Connect a container to labuh-network with DNS aliases
    /// This enables internal DNS resolution like: container-name.labuh, service-name.stack-name.labuh
    pub async fn connect_container_with_alias(
        &self,
        container_id: &str,
        aliases: Vec<String>,
    ) -> Result<()> {
        let docker = &self.container_service.docker;

        let endpoint_config = EndpointSettings {
            aliases: Some(aliases.clone()),
            ..Default::default()
        };

        let config = ConnectNetworkOptions {
            container: container_id,
            endpoint_config,
        };

        docker
            .connect_network(LABUH_NETWORK, config)
            .await
            .map_err(|e| {
                AppError::ContainerRuntime(format!(
                    "Failed to connect container {} with aliases {:?}: {}",
                    container_id, aliases, e
                ))
            })?;

        tracing::info!(
            "Connected container {} to {} with aliases: {:?}",
            container_id,
            LABUH_NETWORK,
            aliases
        );
        Ok(())
    }

    /// Generate DNS aliases for a container
    /// Returns aliases like: ["myapp", "myapp.labuh", "web.mystack.labuh"]
    pub fn generate_container_aliases(
        container_name: &str,
        stack_name: Option<&str>,
        service_name: Option<&str>,
    ) -> Vec<String> {
        let mut aliases = Vec::new();

        // Short name (container name without leading /)
        let short_name = container_name.trim_start_matches('/');
        aliases.push(short_name.to_string());

        // Simple .labuh suffix
        aliases.push(format!("{}.labuh", short_name));

        // If stack/service info is provided, add structured aliases
        if let (Some(stack), Some(service)) = (stack_name, service_name) {
            // service.stack.labuh format
            aliases.push(format!("{}.{}.labuh", service, stack));
        }

        aliases
    }
}
