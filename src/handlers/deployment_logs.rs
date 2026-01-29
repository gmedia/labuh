use axum::{
    extract::{Extension, Path, State},
    routing::get,
    Json, Router,
};
use std::sync::Arc;

use crate::error::Result;
use crate::middleware::auth::CurrentUser;
use crate::models::DeploymentLogResponse;
use crate::services::DeploymentLogService;
use crate::services::StackService;

async fn list_deployment_logs(
    State(state): State<DeploymentLogsState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
) -> Result<Json<Vec<DeploymentLogResponse>>> {
    // Verify user owns this stack
    state
        .stack_service
        .get_stack(&stack_id, &current_user.id)
        .await?;

    let logs = state
        .deployment_log_service
        .list_by_stack(&stack_id, 20)
        .await?;
    let response: Vec<DeploymentLogResponse> = logs.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

#[derive(Clone)]
pub struct DeploymentLogsState {
    pub deployment_log_service: Arc<DeploymentLogService>,
    pub stack_service: Arc<StackService>,
}

pub fn deployment_log_routes(
    deployment_log_service: Arc<DeploymentLogService>,
    stack_service: Arc<StackService>,
) -> Router {
    let state = DeploymentLogsState {
        deployment_log_service,
        stack_service,
    };

    Router::new()
        .route("/{stack_id}/deployments", get(list_deployment_logs))
        .with_state(state)
}
