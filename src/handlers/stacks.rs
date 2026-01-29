use axum::{
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::error::Result;
use crate::middleware::auth::CurrentUser;
use crate::models::{CreateStack, StackResponse};
use crate::services::StackService;
use crate::services::container::ContainerInfo;
use crate::services::stack::{StackHealth, StackLogEntry};

async fn list_stacks(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<StackResponse>>> {
    let stacks = stack_service.list_stacks(&current_user.id).await?;
    let responses: Vec<StackResponse> = stacks.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn get_stack(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<StackResponse>> {
    let stack = stack_service.get_stack(&id, &current_user.id).await?;
    Ok(Json(stack.into()))
}

async fn create_stack(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<CreateStack>,
) -> Result<Json<StackResponse>> {
    let stack = stack_service.create_stack(&request.name, &request.compose_content, &current_user.id).await?;
    Ok(Json(stack.into()))
}

async fn get_stack_containers(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ContainerInfo>>> {
    // Verify user owns the stack
    let _ = stack_service.get_stack(&id, &current_user.id).await?;
    let containers = stack_service.get_stack_containers(&id).await?;
    Ok(Json(containers))
}

async fn start_stack(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    stack_service.start_stack(&id, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "started" })))
}

async fn stop_stack(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    stack_service.stop_stack(&id, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "stopped" })))
}

async fn remove_stack(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    stack_service.remove_stack(&id, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

async fn regenerate_webhook_token(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let token = stack_service.regenerate_webhook_token(&id, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "token": token })))
}

async fn get_stack_health(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<StackHealth>> {
    let health = stack_service.get_stack_health(&id, &current_user.id).await?;
    Ok(Json(health))
}

#[derive(serde::Deserialize)]
struct LogsQuery {
    tail: Option<usize>,
}

async fn get_stack_logs(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<Vec<StackLogEntry>>> {
    let logs = stack_service.get_stack_logs(&id, &current_user.id, query.tail).await?;
    Ok(Json(logs))
}

#[derive(serde::Deserialize)]
struct UpdateStackCompose {
    compose_content: String,
}

async fn update_stack_compose(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(request): Json<UpdateStackCompose>,
) -> Result<Json<serde_json::Value>> {
    stack_service.update_stack_compose(&id, &request.compose_content, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "updated" })))
}

async fn redeploy_stack(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    // Verify ownership
    let _ = stack_service.get_stack(&id, &current_user.id).await?;
    stack_service.redeploy_stack(&id).await?;
    Ok(Json(serde_json::json!({ "status": "redeployed" })))
}

async fn redeploy_service(
    State(stack_service): State<Arc<StackService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((stack_id, service_name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    stack_service.redeploy_service(&stack_id, &service_name, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "redeployed" })))
}

pub fn stack_routes(stack_service: Arc<StackService>) -> Router {
    Router::new()
        .route("/", get(list_stacks))
        .route("/", post(create_stack))
        .route("/{id}", get(get_stack))
        .route("/{id}", delete(remove_stack))
        .route("/{id}/containers", get(get_stack_containers))
        .route("/{id}/health", get(get_stack_health))
        .route("/{id}/logs", get(get_stack_logs))
        .route("/{id}/start", post(start_stack))
        .route("/{id}/stop", post(stop_stack))
        .route("/{id}/redeploy", post(redeploy_stack))
        .route("/{id}/services/{service_name}/redeploy", post(redeploy_service))
        .route("/{id}/compose", axum::routing::put(update_stack_compose))
        .route("/{id}/webhook/regenerate", post(regenerate_webhook_token))
        .with_state(stack_service)
}
