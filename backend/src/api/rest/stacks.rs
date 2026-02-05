use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post},
};
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::models::{
    CreateStack, Stack, StackBackup, StackHealth, StackLogEntry, StackResponse,
};
use crate::error::Result;
use crate::usecase::stack::StackUsecase;

#[derive(serde::Deserialize)]
pub struct ListStacksQuery {
    pub team_id: Option<String>,
}

async fn list_stacks(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ListStacksQuery>,
) -> Result<Json<Vec<StackResponse>>> {
    let stacks: Vec<Stack> = usecase.list_stacks(&current_user.id).await?;
    let responses: Vec<StackResponse> = stacks
        .into_iter()
        .filter(|s| {
            if let Some(ref team_id) = query.team_id {
                s.team_id == *team_id
            } else {
                true
            }
        })
        .map(StackResponse::from)
        .collect();
    Ok(Json(responses))
}

async fn get_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<StackResponse>> {
    let stack: Stack = usecase.get_stack(&id, &current_user.id).await?;
    Ok(Json(stack.into()))
}

async fn create_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<CreateStack>,
) -> Result<Json<StackResponse>> {
    let stack: Stack = usecase
        .create_stack(
            &request.name,
            &request.compose_content,
            &current_user.id,
            &request.team_id,
        )
        .await?;
    Ok(Json(stack.into()))
}

async fn get_stack_containers(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::domain::runtime::ContainerInfo>>> {
    let stack: Stack = usecase.get_stack(&id, &current_user.id).await?;
    let containers = usecase.get_stack_containers(&stack.id).await?;
    Ok(Json(containers))
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

async fn build_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let _: () = usecase.build_stack(&id, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "build_triggered" })))
}

async fn redeploy_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let _stack = usecase.get_stack(&id, &current_user.id).await?;
    let _: () = usecase.redeploy_stack(&id).await?;
    Ok(Json(serde_json::json!({ "status": "redeployed" })))
}

async fn rollback_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let _: () = usecase.rollback_stack(&id, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "rolled_back" })))
}

#[derive(serde::Deserialize)]
struct UpdateStackAutomation {
    cron_schedule: Option<String>,
    health_check_path: Option<String>,
    health_check_interval: i32,
}

async fn update_stack_automation(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(request): Json<UpdateStackAutomation>,
) -> Result<Json<serde_json::Value>> {
    let _: () = usecase
        .update_automation(
            &id,
            &current_user.id,
            request.cron_schedule,
            request.health_check_path,
            request.health_check_interval,
        )
        .await?;
    Ok(Json(serde_json::json!({ "status": "updated" })))
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

#[derive(serde::Deserialize)]
struct ScaleServiceRequest {
    replicas: u64,
}

async fn scale_service(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((stack_id, service_name)): Path<(String, String)>,
    Json(request): Json<ScaleServiceRequest>,
) -> Result<Json<serde_json::Value>> {
    let _: () = usecase
        .scale_service(&stack_id, &service_name, request.replicas, &current_user.id)
        .await?;
    Ok(Json(serde_json::json!({ "status": "scaled" })))
}

async fn build_service(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((stack_id, service_name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let _: () = usecase
        .build_service(&stack_id, &service_name, &current_user.id)
        .await?;
    Ok(Json(serde_json::json!({ "status": "build_triggered" })))
}

async fn get_stack_backup(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<StackBackup>> {
    let backup = usecase.get_stack_backup(&id, &current_user.id).await?;
    Ok(Json(backup))
}

#[derive(serde::Deserialize)]
struct RestoreStackRequest {
    team_id: String,
    backup: StackBackup,
}

async fn restore_stack(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<RestoreStackRequest>,
) -> Result<Json<StackResponse>> {
    let stack = usecase
        .restore_stack(request.backup, &current_user.id, &request.team_id)
        .await?;
    Ok(Json(stack.into()))
}

#[derive(serde::Deserialize)]
struct CreateStackFromGit {
    name: String,
    team_id: String,
    git_url: String,
    git_branch: String,
    compose_path: String,
    env_vars: Option<std::collections::HashMap<String, String>>,
}

async fn create_stack_from_git(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<CreateStackFromGit>,
) -> Result<Json<StackResponse>> {
    let stack = usecase
        .create_stack_from_git(
            &request.name,
            &request.git_url,
            &request.git_branch,
            &request.compose_path,
            &current_user.id,
            &request.team_id,
            request.env_vars,
        )
        .await?;
    Ok(Json(stack.into()))
}

async fn sync_git(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    usecase.sync_git(&id, &current_user.id).await?;
    Ok(Json(serde_json::json!({ "status": "synced" })))
}

async fn build_logs_stream(
    State(usecase): State<Arc<StackUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<
    axum::response::Sse<
        impl tokio_stream::Stream<
            Item = std::result::Result<axum::response::sse::Event, std::convert::Infallible>,
        >,
    >,
> {
    // Verify ownership
    let _stack = usecase.get_stack(&id, &current_user.id).await?;

    let rx = usecase.subscribe_build_logs();
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx);

    let filtered_stream = tokio_stream::StreamExt::filter_map(stream, move |res| match res {
        Ok(msg) if msg.stack_id == id => {
            let json = serde_json::to_string(&msg).ok()?;
            Some(std::result::Result::Ok(
                axum::response::sse::Event::default().data(json),
            ))
        }
        _ => None,
    });

    Ok(axum::response::Sse::new(filtered_stream))
}

pub fn stack_routes(usecase: Arc<StackUsecase>) -> Router {
    Router::new()
        .route("/", get(list_stacks))
        .route("/", post(create_stack))
        .route("/git", post(create_stack_from_git))
        .route("/restore", post(restore_stack))
        .route("/{id}", get(get_stack))
        .route("/{id}", delete(remove_stack))
        .route("/{id}/containers", get(get_stack_containers))
        .route("/{id}/health", get(get_stack_health))
        .route("/{id}/logs", get(get_stack_logs))
        .route("/{id}/build-logs", get(build_logs_stream))
        .route("/{id}/start", post(start_stack))
        .route("/{id}/stop", post(stop_stack))
        .route("/{id}/redeploy", post(redeploy_stack))
        .route("/{id}/build", post(build_stack))
        .route("/{id}/backup", get(get_stack_backup))
        .route("/{id}/git/sync", post(sync_git))
        .route(
            "/{id}/services/{service_name}/redeploy",
            post(redeploy_service),
        )
        .route("/{id}/services/{service_name}/build", post(build_service))
        .route("/{id}/services/{service_name}/scale", post(scale_service))
        .route("/{id}/compose", axum::routing::put(update_stack_compose))
        .route("/{id}/webhook/regenerate", post(regenerate_webhook_token))
        .route(
            "/{id}/automation",
            axum::routing::put(update_stack_automation),
        )
        .route("/{id}/rollback", post(rollback_stack))
        .with_state(usecase)
}
