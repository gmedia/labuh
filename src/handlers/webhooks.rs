use std::sync::Arc;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    error::Result,
    services::stack::StackService,
    services::deployment_log::DeploymentLogService,
    models::CreateDeploymentLog,
};

#[derive(Clone)]
pub struct WebhookState {
    pub stack_service: Arc<StackService>,
    pub deployment_log_service: Arc<DeploymentLogService>,
}

#[derive(serde::Deserialize)]
pub struct WebhookQuery {
    pub service: Option<String>,
}

pub async fn trigger_deploy(
    State(state): State<WebhookState>,
    axum::extract::Query(query): axum::extract::Query<WebhookQuery>,
    Path((stack_id, token)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // 1. Validate token and get stack
    let stack = state.stack_service.validate_webhook_token(&stack_id, &token).await?;

    // 2. Create deployment log entry
    let deployment_log = state.deployment_log_service.create(CreateDeploymentLog {
        stack_id: stack.id.clone(),
        trigger_type: "webhook".to_string(),
    }).await?;

    // 3. Trigger deployment
    // If service query param is present, only redeploy that specific service
    let result = if let Some(service_name) = &query.service {
        state.stack_service.redeploy_service(&stack.id, service_name, &stack.user_id).await
    } else {
        state.stack_service.redeploy_stack(&stack.id).await
    };

    match result {
        Ok(_) => {
            // Log success (redeploy doesn't return container ID, it manages multiple)
            state.deployment_log_service.update_status(
                &deployment_log.id,
                "success",
                Some("Stack redeployed successfully"),
            ).await?;

            Ok(Json(json!({
                "status": "success",
                "message": "Deployment triggered successfully",
                "deployment_id": deployment_log.id
            })))
        }
        Err(e) => {
            // Log failure
            state.deployment_log_service.update_status(
                &deployment_log.id,
                "failed",
                Some(&format!("Deployment failed: {}", e)),
            ).await?;

            Err(e)
        }
    }
}
