use axum::{
    extract::{Extension, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::app_state::AppState;
use crate::error::Result;

#[derive(Serialize)]
pub struct TopologyNode {
    pub id: String,
    pub label: String,
    pub r#type: String, // container | network | service
    pub metadata: serde_json::Value,
}

#[derive(Serialize)]
pub struct TopologyEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

#[derive(Serialize)]
pub struct NetworkTopology {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
}

async fn get_network_topology(
    State(state): State<Arc<AppState>>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<NetworkTopology>> {
    let runtime = &state.runtime;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // 1. Fetch Networks
    let networks = runtime.list_networks().await?;
    let mut network_name_to_id = std::collections::HashMap::new();

    for net in networks {
        network_name_to_id.insert(net.name.clone(), net.id.clone());

        nodes.push(TopologyNode {
            id: net.id.clone(),
            label: net.name.clone(),
            r#type: "network".to_string(),
            metadata: serde_json::json!({
                "driver": net.driver,
                "scope": net.scope,
            }),
        });
    }

    // 2. Fetch Containers
    let containers = runtime.list_containers(true).await?;

    for container in containers {
        nodes.push(TopologyNode {
            id: container.id.clone(),
            label: container.names.first().cloned().unwrap_or_default().replace("/", ""),
            r#type: "container".to_string(),
            metadata: serde_json::json!({
                "image": container.image,
                "state": container.state,
                "status": container.status,
            }),
        });

        // Add edges from container to networks
        for (net_name, endpoint) in container.networks {
            if let Some(net_id) = network_name_to_id.get(&net_name) {
                edges.push(TopologyEdge {
                    from: container.id.clone(),
                    to: net_id.clone(),
                    label: Some(endpoint.ipv4_address),
                });
            }
        }
    }

    Ok(Json(NetworkTopology { nodes, edges }))
}

pub fn network_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/topology", get(get_network_topology))
        .with_state(state)
}
