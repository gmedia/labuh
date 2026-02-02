use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::runtime::{ImageInfo, ImageInspect, RuntimePort};
use crate::error::Result;
use crate::usecase::registry::RegistryUsecase;

#[derive(Clone)]
pub struct ImageState {
    pub runtime: Arc<dyn RuntimePort>,
    pub registry_usecase: Arc<RegistryUsecase>,
}

#[derive(Deserialize)]
pub struct PullImageRequest {
    image: String,
    team_id: String,
}

async fn list_images(State(state): State<ImageState>) -> Result<Json<Vec<ImageInfo>>> {
    let images = state.runtime.list_images().await?;
    Ok(Json(images))
}

async fn inspect_image(
    State(state): State<ImageState>,
    Path(id): Path<String>,
) -> Result<Json<ImageInspect>> {
    let inspect = state.runtime.inspect_image(&id).await?;
    Ok(Json(inspect))
}

async fn pull_image(
    State(state): State<ImageState>,
    Extension(user): Extension<CurrentUser>,
    Json(request): Json<PullImageRequest>,
) -> Result<Json<serde_json::Value>> {
    let creds = state
        .registry_usecase
        .get_credentials_for_image(&user.id, &request.team_id, &request.image)
        .await?;
    state.runtime.pull_image(&request.image, creds).await?;
    Ok(Json(
        serde_json::json!({ "status": "pulled", "image": request.image }),
    ))
}

async fn remove_image(
    State(state): State<ImageState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    state.runtime.remove_image(&id, false).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

pub fn image_routes(
    runtime: Arc<dyn RuntimePort>,
    registry_usecase: Arc<RegistryUsecase>,
) -> Router {
    let state = ImageState {
        runtime,
        registry_usecase,
    };
    Router::new()
        .route("/", get(list_images))
        .route("/pull", post(pull_image))
        .route("/{id}", delete(remove_image))
        .route("/{id}/inspect", get(inspect_image))
        .with_state(state)
}
