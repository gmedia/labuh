use crate::usecase::node::NodeUsecase;
use ax_auth::CurrentUser;
use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::{get, post},
};
use std::sync::Arc;

pub async fn list_nodes(
    State(usecase): State<Arc<NodeUsecase>>,
    Extension(_user): Extension<CurrentUser>,
) -> Json<serde_json::Value> {
    match usecase.list_nodes().await {
        Ok(nodes) => Json(serde_json::json!(nodes)),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn inspect_node(
    State(usecase): State<Arc<NodeUsecase>>,
    Extension(_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match usecase.inspect_node(&id).await {
        Ok(node) => Json(serde_json::json!(node)),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn swarm_info(
    State(usecase): State<Arc<NodeUsecase>>,
    Extension(_user): Extension<CurrentUser>,
) -> Json<serde_json::Value> {
    match usecase.is_swarm_enabled().await {
        Ok(enabled) => Json(serde_json::json!({ "enabled": enabled })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(serde::Deserialize)]
pub struct SwarmInitBody {
    pub listen_addr: String,
}

#[derive(serde::Deserialize)]
pub struct SwarmJoinBody {
    pub listen_addr: String,
    pub remote_addr: String,
    pub token: String,
}

pub async fn init_swarm(
    State(usecase): State<Arc<NodeUsecase>>,
    Extension(_user): Extension<CurrentUser>,
    Json(body): Json<SwarmInitBody>,
) -> Json<serde_json::Value> {
    match usecase.init_swarm(&body.listen_addr).await {
        Ok(token) => Json(serde_json::json!({ "token": token })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn join_swarm(
    State(usecase): State<Arc<NodeUsecase>>,
    Extension(_user): Extension<CurrentUser>,
    Json(body): Json<SwarmJoinBody>,
) -> Json<serde_json::Value> {
    match usecase
        .join_swarm(&body.listen_addr, &body.remote_addr, &body.token)
        .await
    {
        Ok(_) => Json(serde_json::json!({ "status": "ok" })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn get_tokens(
    State(usecase): State<Arc<NodeUsecase>>,
    Extension(_user): Extension<CurrentUser>,
) -> Json<serde_json::Value> {
    match usecase.get_tokens().await {
        Ok(tokens) => Json(serde_json::json!(tokens)),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub fn node_routes(usecase: Arc<NodeUsecase>) -> Router {
    Router::new()
        .route("/", get(list_nodes))
        .route("/swarm", get(swarm_info))
        .route("/swarm/init", post(init_swarm))
        .route("/swarm/join", post(join_swarm))
        .route("/swarm/tokens", get(get_tokens))
        .route("/{id}", get(inspect_node))
        .with_state(usecase)
}

mod ax_auth {
    pub use crate::api::middleware::auth::CurrentUser;
}
