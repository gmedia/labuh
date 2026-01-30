use axum::{
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::models::{CreateStack, StackHealth, StackLogEntry, StackResponse};
use crate::error::Result;
use crate::usecase::stack::StackUsecase;

async fn list_stacks(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<StackResponse>>> {
    let stacks: Vec<crate::domain::models::Stack> = usecase.list_stacks(&current_user.id).await?;
    let responses: Vec<StackResponse> = stacks.into_iter().map(StackResponse::from).collect();
    Ok(Json(responses))
}

async fn get_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<StackResponse>> {
    let stack: crate::domain::models::Stack = usecase.get_stack(&id, &current_user.id).await?;
    Ok(Json(stack.into()))
}

async fn create_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<CreateStack>,
) -> Result<Json<StackResponse>> {
    let stack: crate::domain::models::Stack = usecase
        .create_stack(&request.name, &request.compose_content, &current_user.id)
        .await?;
    Ok(Json(stack.into()))
}

async fn get_stack_containers(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::domain::runtime::ContainerInfo>>> {
    let _stack: crate::domain::models::Stack = usecase.get_stack(&id, &current_user.id).await?;
    // For now, placeholder as we need a public method in usecase to get containers
    Ok(Json(vec![]))
}

async fn start_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let _: () = usecase.start_stack(&id, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "started" })))
}

async fn stop_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let _: () = usecase.stop_stack(&id, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "stopped" })))
}

async fn remove_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let _: () = usecase.remove_stack(&id, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

async fn regenerate_webhook_token(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let token: String = usecase
        .regenerate_webhook_token(&id, &current_user.id)
        .await?;
    Ok(Json(serde_json::json!({ "token": token })))
}

async fn get_stack_health(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<StackHealth>> {
    let health: StackHealth = usecase.get_stack_health(&id, &current_user.id).await?;
    Ok(Json(health))
}

#[derive(serde::Deserialize)]
struct LogsQuery {
    tail: Option<usize>,
}

async fn get_stack_logs(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<Vec<StackLogEntry>>> {
    let logs: Vec<StackLogEntry> = usecase
        .get_stack_logs(&id, &current_user.id, query.tail)
        .await?;
    Ok(Json(logs))
}

#[derive(serde::Deserialize)]
struct UpdateStackCompose {
    compose_content: String,
}

async fn update_stack_compose(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(request): Json<UpdateStackCompose>,
) -> Result<Json<serde_json::Value>> {
    let _: () = usecase
        .update_stack_compose(&id, &request.compose_content, &current_user.id)
        .await?;
    Ok(Json(serde_json::json!({ "status": "updated" })))
}

async fn redeploy_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let _: () = usecase.redeploy_stack(&id).await?;
    Ok(Json(serde_json::json!({ "status": "redeployed" })))
}

async fn redeploy_service(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((stack_id, service_name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let _: () = usecase
        .redeploy_service(&stack_id, &service_name, &current_user.id)
        .await?;
    Ok(Json(serde_json::json!({ "status": "redeployed" })))
}

pub fn stack_routes(usecase: Arc<StackUsecase>) -> Router {
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
        .route(
            "/{id}/services/{service_name}/redeploy",
            post(redeploy_service),
        )
        .route("/{id}/compose", axum::routing::put(update_stack_compose))
        .route("/{id}/webhook/regenerate", post(regenerate_webhook_token))
        .with_state(usecase)
}
