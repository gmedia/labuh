use crate::domain::runtime::{ContainerConfig, RuntimePort};
use crate::error::Result;
use base64::{Engine as _, engine::general_purpose};
use std::sync::Arc;

const LABUH_NETWORK: &str = "labuh-network";
const TUNNEL_CONTAINER_NAME: &str = "labuh-tunnel";
const TUNNEL_IMAGE: &str = "cloudflare/cloudflared:latest";

pub struct TunnelManager {
    runtime: Arc<dyn RuntimePort>,
}

impl TunnelManager {
    pub fn new(runtime: Arc<dyn RuntimePort>) -> Self {
        Self { runtime }
    }

    /// Ensure the cloudflared tunnel container is running with the given token
    pub async fn ensure_tunnel(&self, token: &str) -> Result<()> {
        // Check if container already exists
        let containers = self.runtime.list_containers(true).await?;
        let existing = containers
            .iter()
            .find(|c| c.names.iter().any(|n| n.contains(TUNNEL_CONTAINER_NAME)));

        if let Some(container) = existing {
            if container.state == "running" {
                tracing::debug!(
                    "Tunnel container {} is already running",
                    TUNNEL_CONTAINER_NAME
                );
                return Ok(());
            } else {
                tracing::info!(
                    "Starting existing tunnel container {}",
                    TUNNEL_CONTAINER_NAME
                );
                return self.runtime.start_container(&container.id).await;
            }
        }

        // Create and start new container
        tracing::info!("Deploying new tunnel container: {}", TUNNEL_CONTAINER_NAME);

        self.runtime.pull_image(TUNNEL_IMAGE, None).await?;

        // Ensure network exists
        self.runtime.ensure_network(LABUH_NETWORK).await?;

        let labels = vec![
            ("labuh.managed".to_string(), "true".to_string()),
            ("labuh.service".to_string(), "tunnel".to_string()),
        ]
        .into_iter()
        .collect();

        let config = ContainerConfig {
            name: TUNNEL_CONTAINER_NAME.to_string(),
            image: TUNNEL_IMAGE.to_string(),
            env: None,
            cmd: Some(vec![
                "tunnel".to_string(),
                "run".to_string(),
                "--token".to_string(),
                token.to_string(),
            ]),
            ports: None,
            volumes: None,
            labels: Some(labels),
            cpu_limit: None,
            memory_limit: None,
            network_mode: Some(LABUH_NETWORK.to_string()),
            networks: None,
            extra_hosts: None,
            restart_policy: Some("always".to_string()),
        };

        let id = self.runtime.create_container(config).await?;

        self.runtime.start_container(&id).await?;

        // Ensure network connection (redundant with network_mode but safe)
        // self.runtime.connect_network(&id, LABUH_NETWORK).await?;

        Ok(())
    }

    /// Helper to extract tunnel ID from a Cloudflare Tunnel token
    pub fn extract_tunnel_id(token: &str) -> Option<String> {
        let decoded = general_purpose::STANDARD.decode(token).ok()?;
        let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
        json["t"].as_str().map(|s| s.to_string())
    }
}
