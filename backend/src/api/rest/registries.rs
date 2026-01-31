use axum::{
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::models::{CreateRegistryCredential, RegistryCredentialResponse};
use crate::error::Result;
use crate::usecase::registry::RegistryUsecase;

#[derive(serde::Deserialize)]
struct TeamQuery {
    team_id: String,
}

async fn list_credentials(
    State(usecase): State<Arc<RegistryUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<TeamQuery>,
) -> Result<Json<Vec<RegistryCredentialResponse>>> {
    let credentials = usecase
        .list_credentials(&query.team_id, &current_user.id)
        .await?;
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
            &request.team_id,
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
    Extension(_current_user): Extension<CurrentUser>,
    Path((_team_id, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    usecase.remove_credential(&id, &_team_id).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

pub fn registry_routes(usecase: Arc<RegistryUsecase>) -> Router {
    Router::new()
        .route("/", get(list_credentials))
        .route("/", post(add_credential))
        .route("/{team_id}/{id}", delete(remove_credential))
        .with_state(usecase)
}
