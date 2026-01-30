//! Environment handlers for stack environment variables API

use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;

use crate::error::Result;
use crate::middleware::auth::CurrentUser;
use crate::services::environment::{EnvironmentService, StackEnvVarResponse};
use crate::services::StackService;

#[derive(serde::Deserialize)]
pub struct SetEnvVar {
    #[serde(default)]
    pub container_name: String,
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub is_secret: bool,
}

#[derive(serde::Deserialize)]
pub struct BulkSetEnvVars {
    #[serde(default)]
    pub container_name: String,
    pub vars: Vec<SetEnvVarEntry>,
}

#[derive(serde::Deserialize)]
pub struct SetEnvVarEntry {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub is_secret: bool,
}

#[derive(serde::Deserialize)]
pub struct DeleteEnvVarQuery {
    #[serde(default)]
    pub container_name: String,
}

async fn list_env_vars(
    State((env_service, stack_service)): State<(Arc<EnvironmentService>, Arc<StackService>)>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
) -> Result<Json<Vec<StackEnvVarResponse>>> {
    // Verify user owns the stack
    stack_service.get_stack(&stack_id, &current_user.id).await?;

    let vars = env_service.list(&stack_id).await?;
    let responses: Vec<StackEnvVarResponse> = vars.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn set_env_var(
    State((env_service, stack_service)): State<(Arc<EnvironmentService>, Arc<StackService>)>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
    Json(request): Json<SetEnvVar>,
) -> Result<Json<StackEnvVarResponse>> {
    // Verify user owns the stack
    stack_service.get_stack(&stack_id, &current_user.id).await?;

    let var = env_service
        .set(
            &stack_id,
            &request.container_name,
            &request.key,
            &request.value,
            request.is_secret,
        )
        .await?;
    Ok(Json(var.into()))
}

async fn bulk_set_env_vars(
    State((env_service, stack_service)): State<(Arc<EnvironmentService>, Arc<StackService>)>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
    Json(request): Json<BulkSetEnvVars>,
) -> Result<Json<Vec<StackEnvVarResponse>>> {
    // Verify user owns the stack
    stack_service.get_stack(&stack_id, &current_user.id).await?;

    let vars: Vec<(String, String, bool)> = request
        .vars
        .into_iter()
        .map(|v| (v.key, v.value, v.is_secret))
        .collect();

    let results = env_service
        .bulk_set(&stack_id, &request.container_name, vars)
        .await?;
    let responses: Vec<StackEnvVarResponse> = results.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn delete_env_var(
    State((env_service, stack_service)): State<(Arc<EnvironmentService>, Arc<StackService>)>,
    Extension(current_user): Extension<CurrentUser>,
    Path((stack_id, key)): Path<(String, String)>,
    axum::extract::Query(query): axum::extract::Query<DeleteEnvVarQuery>,
) -> Result<Json<serde_json::Value>> {
    // Verify user owns the stack
    stack_service.get_stack(&stack_id, &current_user.id).await?;

    env_service
        .delete(&stack_id, &query.container_name, &key)
        .await?;
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

pub fn environment_routes(
    env_service: Arc<EnvironmentService>,
    stack_service: Arc<StackService>,
) -> Router {
    Router::new()
        .route("/{stack_id}/env", get(list_env_vars))
        .route("/{stack_id}/env", post(set_env_var))
        .route("/{stack_id}/env/bulk", put(bulk_set_env_vars))
        .route("/{stack_id}/env/{key}", delete(delete_env_var))
        .with_state((env_service, stack_service))
}
