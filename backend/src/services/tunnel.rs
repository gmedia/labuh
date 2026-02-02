use bollard::container::{Config, CreateContainerOptions};
use bollard::models::HostConfig;
use std::sync::Arc;

use crate::error::{AppError, Result};
use crate::services::network::{NetworkService, LABUH_NETWORK};
use crate::services::ContainerService;

#[allow(dead_code)]
pub struct TunnelService {
    container_service: Arc<ContainerService>,
    network_service: Arc<NetworkService>,
}

#[allow(dead_code)]
impl TunnelService {
    pub fn new(
        container_service: Arc<ContainerService>,
        network_service: Arc<NetworkService>,
    ) -> Self {
        Self {
            container_service,
            network_service,
        }
    }

    /// Ensure the cloudflared tunnel container is running with the given token
    pub async fn ensure_tunnel(&self, token: &str) -> Result<()> {
        let container_name = "labuh-tunnel";

        // Check if container already exists
        let containers = self.container_service.list_containers(true).await?;
        let existing = containers
            .iter()
            .find(|c| c.names.iter().any(|n| n == &format!("/{}", container_name)));

        if let Some(container) = existing {
            if container.state == "running" {
                tracing::debug!("Tunnel container {} is already running", container_name);
                return Ok(());
            } else {
                tracing::info!("Starting existing tunnel container {}", container_name);
                return self.container_service.start_container(&container.id).await;
            }
        }

        // Create and start new container
        tracing::info!("Deploying new tunnel container: {}", container_name);

        let image = "cloudflare/cloudflared:latest";
        self.container_service.pull_image(image).await?;

        let config = Config {
            image: Some(image.to_string()),
            cmd: Some(vec![
                "tunnel".to_string(),
                "run".to_string(),
                "--token".to_string(),
                token.to_string(),
            ]),
            host_config: Some(HostConfig {
                network_mode: Some(LABUH_NETWORK.to_string()),
                restart_policy: Some(bollard::models::RestartPolicy {
                    name: Some(bollard::models::RestartPolicyNameEnum::ALWAYS),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            labels: Some(
                vec![
                    ("labuh.managed".to_string(), "true".to_string()),
                    ("labuh.service".to_string(), "tunnel".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: container_name,
            ..Default::default()
        };

        let resp = self
            .container_service
            .docker
            .create_container(Some(options), config)
            .await
            .map_err(|e| {
                AppError::ContainerRuntime(format!("Failed to create tunnel container: {}", e))
            })?;

        self.container_service.start_container(&resp.id).await?;

        // Ensure network connection (though network_mode should handle it)
        let _ = self.network_service.connect_container(container_name).await;

        Ok(())
    }

    /// Stop and remove the tunnel container
    pub async fn stop_tunnel(&self) -> Result<()> {
        let container_name = "labuh-tunnel";
        let _ = self.container_service.stop_container(container_name).await;
        let _ = self
            .container_service
            .remove_container(container_name, true)
            .await;
        Ok(())
    }
}
