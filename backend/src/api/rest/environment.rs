use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::models::{BulkSetEnvVarRequest, SetEnvVarRequest, StackEnvVarResponse};
use crate::error::Result;
use crate::usecase::environment::EnvironmentUsecase;
use crate::usecase::stack::StackUsecase;

async fn list_env_vars(
    State((env_usecase, stack_usecase)): State<(Arc<EnvironmentUsecase>, Arc<StackUsecase>)>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
) -> Result<Json<Vec<StackEnvVarResponse>>> {
    // Verify user owns the stack
    stack_usecase.get_stack(&stack_id, &current_user.id).await?;

    let vars = env_usecase.list_vars(&stack_id).await?;
    Ok(Json(vars))
}

async fn set_env_var(
    State((env_usecase, stack_usecase)): State<(Arc<EnvironmentUsecase>, Arc<StackUsecase>)>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
    Json(request): Json<SetEnvVarRequest>,
) -> Result<Json<StackEnvVarResponse>> {
    // Verify user owns the stack
    stack_usecase.get_stack(&stack_id, &current_user.id).await?;

    let var = env_usecase
        .set_var(
            &stack_id,
            &request.container_name,
            &request.key,
            &request.value,
            request.is_secret,
        )
        .await?;
    Ok(Json(var))
}

async fn bulk_set_env_vars(
    State((env_usecase, stack_usecase)): State<(Arc<EnvironmentUsecase>, Arc<StackUsecase>)>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
    Json(request): Json<BulkSetEnvVarRequest>,
) -> Result<Json<Vec<StackEnvVarResponse>>> {
    // Verify user owns the stack
    stack_usecase.get_stack(&stack_id, &current_user.id).await?;

    let vars: Vec<(String, String, bool)> = request
        .vars
        .into_iter()
        .map(|v| (v.key, v.value, v.is_secret))
        .collect();

    let results = env_usecase
        .bulk_set(&stack_id, &request.container_name, vars)
        .await?;
    Ok(Json(results))
}

#[derive(serde::Deserialize)]
pub struct DeleteEnvVarQuery {
    #[serde(default)]
    pub container_name: String,
}

async fn delete_env_var(
    State((env_usecase, stack_usecase)): State<(Arc<EnvironmentUsecase>, Arc<StackUsecase>)>,
    Extension(current_user): Extension<CurrentUser>,
    Path((stack_id, key)): Path<(String, String)>,
    axum::extract::Query(query): axum::extract::Query<DeleteEnvVarQuery>,
) -> Result<Json<serde_json::Value>> {
    // Verify user owns the stack
    stack_usecase.get_stack(&stack_id, &current_user.id).await?;

    env_usecase
        .delete_var(&stack_id, &query.container_name, &key)
        .await?;
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

pub fn environment_routes(
    env_usecase: Arc<EnvironmentUsecase>,
    stack_usecase: Arc<StackUsecase>,
) -> Router {
    Router::new()
        .route("/{stack_id}/env", get(list_env_vars))
        .route("/{stack_id}/env", post(set_env_var))
        .route("/{stack_id}/env/bulk", put(bulk_set_env_vars))
        .route("/{stack_id}/env/{key}", delete(delete_env_var))
        .with_state((env_usecase, stack_usecase))
}
