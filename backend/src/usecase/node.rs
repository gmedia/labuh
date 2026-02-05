use crate::domain::runtime::{RuntimePort, SwarmNode, SwarmTokens};
use crate::error::Result;
use std::sync::Arc;

pub struct NodeUsecase {
    runtime: Arc<dyn RuntimePort>,
}

impl NodeUsecase {
    pub fn new(runtime: Arc<dyn RuntimePort>) -> Self {
        Self { runtime }
    }

    pub async fn list_nodes(&self) -> Result<Vec<SwarmNode>> {
        self.runtime.list_nodes().await
    }

    pub async fn inspect_node(&self, id: &str) -> Result<SwarmNode> {
        self.runtime.inspect_node(id).await
    }

    pub async fn is_swarm_enabled(&self) -> Result<bool> {
        self.runtime.is_swarm_enabled().await
    }

    pub async fn init_swarm(&self, listen_addr: &str) -> Result<String> {
        let token = self.runtime.swarm_init(listen_addr).await?;

        // After swarm init, migrate labuh-network to overlay for compatibility
        tracing::info!("Swarm initialized, migrating labuh-network to overlay...");
        if let Err(e) = self
            .runtime
            .migrate_network_to_overlay("labuh-network")
            .await
        {
            tracing::warn!(
                "Failed to migrate labuh-network to overlay: {}. You may need to manually recreate it.",
                e
            );
        }

        // Reconnect Caddy to the new network
        if let Err(e) = self
            .runtime
            .connect_network("labuh-caddy", "labuh-network")
            .await
        {
            tracing::warn!("Failed to reconnect Caddy to labuh-network: {}", e);
        }

        Ok(token)
    }

    pub async fn join_swarm(
        &self,
        listen_addr: &str,
        remote_addr: &str,
        token: &str,
    ) -> Result<()> {
        self.runtime
            .swarm_join(listen_addr, remote_addr, token)
            .await?;

        // After joining swarm, migrate labuh-network to overlay for compatibility
        tracing::info!("Joined swarm, migrating labuh-network to overlay...");
        if let Err(e) = self
            .runtime
            .migrate_network_to_overlay("labuh-network")
            .await
        {
            tracing::warn!(
                "Failed to migrate labuh-network to overlay: {}. You may need to manually recreate it.",
                e
            );
        }

        // Reconnect Caddy to the new network
        if let Err(e) = self
            .runtime
            .connect_network("labuh-caddy", "labuh-network")
            .await
        {
            tracing::warn!("Failed to reconnect Caddy to labuh-network: {}", e);
        }

        Ok(())
    }

    pub async fn get_tokens(&self) -> Result<SwarmTokens> {
        self.runtime.get_swarm_tokens().await
    }
}
