use axum::{
    extract::{Extension, Path, Query, State},
    routing::get,
    Json, Router,
};
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::models::DeploymentLogResponse;
use crate::error::Result;
use crate::usecase::deployment_log::DeploymentLogUsecase;
use crate::usecase::stack::StackUsecase;

#[derive(serde::Deserialize)]
pub struct LogsQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_limit() -> i32 {
    10
}

async fn list_deployment_logs(
    State((usecase, stack_usecase)): State<(Arc<DeploymentLogUsecase>, Arc<StackUsecase>)>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<Vec<DeploymentLogResponse>>> {
    // Verify user owns the stack
    stack_usecase.get_stack(&stack_id, &current_user.id).await?;

    let logs = usecase.list_logs(&stack_id, query.limit).await?;
    Ok(Json(logs))
}

async fn get_deployment_log(
    State((usecase, _stack_usecase)): State<(Arc<DeploymentLogUsecase>, Arc<StackUsecase>)>,
    Extension(_current_user): Extension<CurrentUser>,
    Path((_stack_id, log_id)): Path<(String, String)>,
) -> Result<Json<DeploymentLogResponse>> {
    let log = usecase.get_log(&log_id).await?;
    Ok(Json(log))
}

pub fn deployment_log_routes(
    usecase: Arc<DeploymentLogUsecase>,
    stack_usecase: Arc<StackUsecase>,
) -> Router {
    Router::new()
        .route("/{stack_id}/deployments", get(list_deployment_logs))
        .route("/{stack_id}/deployments/{log_id}", get(get_deployment_log))
        .with_state((usecase, stack_usecase))
}
