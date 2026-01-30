use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::{
    error::Result, usecase::deployment_log::DeploymentLogUsecase, usecase::stack::StackUsecase,
};

#[derive(Clone)]
pub struct WebhookState {
    pub stack_usecase: Arc<StackUsecase>,
    pub deployment_log_usecase: Arc<DeploymentLogUsecase>,
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
    let stack = state
        .stack_usecase
        .validate_webhook_token(&stack_id, &token)
        .await?;

    // 2. Create deployment log entry
    let deployment_log = state
        .deployment_log_usecase
        .create_log(&stack.id, "webhook")
        .await?;

    // 3. Trigger deployment
    // If service query param is present, only redeploy that specific service
    let result = if let Some(service_name) = &query.service {
        state
            .stack_usecase
            .redeploy_service(&stack.id, service_name, &stack.user_id)
            .await
    } else {
        state.stack_usecase.redeploy_stack(&stack.id).await
    };

    match result {
        Ok(_) => {
            // Log success
            state
                .deployment_log_usecase
                .update_status(
                    &deployment_log.id,
                    "success",
                    Some("Stack redeployed successfully"),
                )
                .await?;

            Ok(Json(json!({
                "status": "success",
                "message": "Deployment triggered successfully",
                "deployment_id": deployment_log.id
            })))
        }
        Err(e) => {
            // Log failure
            state
                .deployment_log_usecase
                .update_status(
                    &deployment_log.id,
                    "failed",
                    Some(&format!("Deployment failed: {}", e)),
                )
                .await?;

            Err(e)
        }
    }
}
