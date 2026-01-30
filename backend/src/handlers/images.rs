use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::Result;
use crate::services::container::{ImageInfo, ImageInspect};
use crate::services::ContainerService;

#[derive(Deserialize)]
pub struct PullImageRequest {
    image: String,
}

async fn list_images(
    State(container_service): State<Arc<ContainerService>>,
) -> Result<Json<Vec<ImageInfo>>> {
    let images = container_service.list_images().await?;
    Ok(Json(images))
}

async fn inspect_image(
    State(container_service): State<Arc<ContainerService>>,
    Path(id): Path<String>,
) -> Result<Json<ImageInspect>> {
    let inspect = container_service.inspect_image(&id).await?;
    Ok(Json(inspect))
}

async fn pull_image(
    State(container_service): State<Arc<ContainerService>>,
    Json(request): Json<PullImageRequest>,
) -> Result<Json<serde_json::Value>> {
    container_service.pull_image(&request.image).await?;
    Ok(Json(
        serde_json::json!({ "status": "pulled", "image": request.image }),
    ))
}

async fn remove_image(
    State(container_service): State<Arc<ContainerService>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    container_service.remove_image(&id, false).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

pub fn image_routes(container_service: Arc<ContainerService>) -> Router {
    Router::new()
        .route("/", get(list_images))
        .route("/pull", post(pull_image))
        .route("/{id}", delete(remove_image))
        .route("/{id}/inspect", get(inspect_image))
        .with_state(container_service)
}
