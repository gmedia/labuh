use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::error::Result;
use crate::middleware::auth::CurrentUser;
use crate::models::{CreateRegistryCredential, RegistryCredentialResponse};
use crate::services::RegistryService;

async fn list_credentials(
    State(registry_service): State<Arc<RegistryService>>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<RegistryCredentialResponse>>> {
    let credentials = registry_service.list_credentials(&current_user.id).await?;
    let responses: Vec<RegistryCredentialResponse> =
        credentials.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn add_credential(
    State(registry_service): State<Arc<RegistryService>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<CreateRegistryCredential>,
) -> Result<Json<RegistryCredentialResponse>> {
    let credential = registry_service
        .add_credential(
            &current_user.id,
            &request.name,
            &request.registry_url,
            &request.username,
            &request.password,
        )
        .await?;
    Ok(Json(credential.into()))
}

async fn remove_credential(
    State(registry_service): State<Arc<RegistryService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    registry_service
        .remove_credential(&id, &current_user.id)
        .await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

pub fn registry_routes(registry_service: Arc<RegistryService>) -> Router {
    Router::new()
        .route("/", get(list_credentials))
        .route("/", post(add_credential))
        .route("/{id}", delete(remove_credential))
        .with_state(registry_service)
}
