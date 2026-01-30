use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::models::{CreateRegistryCredential, RegistryCredentialResponse};
use crate::error::Result;
use crate::usecase::registry::RegistryUsecase;

async fn list_credentials(
    State(usecase): State<Arc<RegistryUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<RegistryCredentialResponse>>> {
    let credentials = usecase.list_credentials(&current_user.id).await?;
    Ok(Json(credentials))
}

async fn add_credential(
    State(usecase): State<Arc<RegistryUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<CreateRegistryCredential>,
) -> Result<Json<RegistryCredentialResponse>> {
    let credential = usecase
        .add_credential(
            &current_user.id,
            &request.name,
            &request.registry_url,
            &request.username,
            &request.password,
        )
        .await?;
    Ok(Json(credential))
}

async fn remove_credential(
    State(usecase): State<Arc<RegistryUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    usecase.remove_credential(&id, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

pub fn registry_routes(usecase: Arc<RegistryUsecase>) -> Router {
    Router::new()
        .route("/", get(list_credentials))
        .route("/", post(add_credential))
        .route("/{id}", delete(remove_credential))
        .with_state(usecase)
}
