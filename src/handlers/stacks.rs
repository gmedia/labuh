use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::error::Result;
use crate::middleware::auth::CurrentUser;
use crate::models::{CreateStack, StackResponse};
use crate::services::StackService;
use crate::services::container::ContainerInfo;

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

pub fn stack_routes(stack_service: Arc<StackService>) -> Router {
    Router::new()
        .route("/", get(list_stacks))
        .route("/", post(create_stack))
        .route("/{id}", get(get_stack))
        .route("/{id}", delete(remove_stack))
        .route("/{id}/containers", get(get_stack_containers))
        .route("/{id}/start", post(start_stack))
        .route("/{id}/stop", post(stop_stack))
        .route("/{id}/webhook/regenerate", post(regenerate_webhook_token))
        .with_state(stack_service)
}
