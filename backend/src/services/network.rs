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
}
