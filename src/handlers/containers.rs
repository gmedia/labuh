use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::Result;
use crate::services::container::{ContainerInfo, ContainerStats, CreateContainerRequest, ImageInfo};
use crate::services::ContainerService;

#[derive(Deserialize)]
pub struct ListContainersQuery {
    #[serde(default)]
    all: bool,
}

#[derive(Deserialize)]
pub struct LogsQuery {
    #[serde(default = "default_tail")]
    tail: usize,
}

fn default_tail() -> usize {
    100
}

async fn list_containers(
    State(container_service): State<Arc<ContainerService>>,
    Query(query): Query<ListContainersQuery>,
) -> Result<Json<Vec<ContainerInfo>>> {
    let containers = container_service.list_containers(query.all).await?;
    Ok(Json(containers))
}

async fn create_container(
    State(container_service): State<Arc<ContainerService>>,
    Json(request): Json<CreateContainerRequest>,
) -> Result<Json<serde_json::Value>> {
    let id = container_service.create_container(request).await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn start_container(
    State(container_service): State<Arc<ContainerService>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    container_service.start_container(&id).await?;
    Ok(Json(serde_json::json!({ "status": "started" })))
}

async fn stop_container(
    State(container_service): State<Arc<ContainerService>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    container_service.stop_container(&id).await?;
    Ok(Json(serde_json::json!({ "status": "stopped" })))
}

async fn restart_container(
    State(container_service): State<Arc<ContainerService>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    container_service.restart_container(&id).await?;
    Ok(Json(serde_json::json!({ "status": "restarted" })))
}

async fn remove_container(
    State(container_service): State<Arc<ContainerService>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    container_service.remove_container(&id, false).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

async fn get_container_logs(
    State(container_service): State<Arc<ContainerService>>,
    Path(id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<Vec<String>>> {
    let logs = container_service.get_container_logs(&id, query.tail).await?;
    Ok(Json(logs))
}

async fn get_container_stats(
    State(container_service): State<Arc<ContainerService>>,
    Path(id): Path<String>,
) -> Result<Json<ContainerStats>> {
    let stats = container_service.get_container_stats(&id).await?;
    Ok(Json(stats))
}

pub fn container_routes(container_service: Arc<ContainerService>) -> Router {
    Router::new()
        .route("/", get(list_containers))
        .route("/", post(create_container))
        .route("/{id}/start", post(start_container))
        .route("/{id}/stop", post(stop_container))
        .route("/{id}/restart", post(restart_container))
        .route("/{id}", delete(remove_container))
        .route("/{id}/logs", get(get_container_logs))
        .route("/{id}/stats", get(get_container_stats))
        .with_state(container_service)
}
