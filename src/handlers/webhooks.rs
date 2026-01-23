use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    error::{AppError, Result},
    services::project::ProjectService,
    services::container::ContainerService,
    models::project::Project,
};

#[derive(Clone)]
pub struct WebhookState {
    pub project_service: Arc<ProjectService>,
    pub container_service: Arc<ContainerService>,
}

pub async fn trigger_deploy(
    State(state): State<WebhookState>,
    Path((project_id, token)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // 1. Validate token and get project
    let project = state.project_service.validate_webhook_token(&project_id, &token).await?;

    // 2. Trigger deployment
    // This logic is similar to the manual deploy, but authenticated via token
    // We basically need to pull the latest image and restart the container

    // For now, we'll reuse the deploy logic if possible, or replicate the essential steps.
    // Since ContainerService::deploy_project handles pulling and starting, we can use that.

    // We need to re-fetch the project to ensure we have the latest details (though validate_project returned it)
    // The main issue might be if deploy_project requires a user_id for logging/permission checks that are different here.
    // But since we validated the token, we are authorized.

    // However, ContainerService methods might expect user_id.
    // Let's check ContainerService::deploy_project signature.
    // It is: pub async fn deploy_project(&self, project: &Project) -> Result<String>
    // It takes &Project, so we are good.

    let container_id = state.container_service.deploy_project(&project).await?;

    // 3. Update project status
    state.project_service.update_project_status(&project.id, "running", Some(&container_id)).await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Deployment triggered successfully",
        "container_id": container_id
    })))
}
