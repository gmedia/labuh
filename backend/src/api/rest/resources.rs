use axum::{
    extract::{Extension, Path, Query, State},
    routing::{get, put},
    Json, Router,
};
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::models::resource::{ContainerResource, ResourceMetric, UpdateResourceRequest};
use crate::error::Result;
use crate::usecase::resource::ResourceUsecase;

async fn list_stack_limits(
    State(usecase): State<Arc<ResourceUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
) -> Result<Json<Vec<ContainerResource>>> {
    let limits = usecase.get_limits(&stack_id, &current_user.id).await?;
    Ok(Json(limits))
}

async fn update_service_limits(
    State(usecase): State<Arc<ResourceUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((stack_id, service_name)): Path<(String, String)>,
    Json(request): Json<UpdateResourceRequest>,
) -> Result<Json<serde_json::Value>> {
    usecase
        .update_limits(
            &stack_id,
            &service_name,
            &current_user.id,
            request.cpu_limit,
            request.memory_limit,
        )
        .await?;
    Ok(Json(serde_json::json!({ "status": "updated" })))
}

#[derive(serde::Deserialize)]
struct MetricsQuery {
    range: Option<String>,
}

async fn get_stack_metrics(
    State(usecase): State<Arc<ResourceUsecase>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<Vec<ResourceMetric>>> {
    let range = query.range.unwrap_or_else(|| "1h".to_string());
    let metrics = usecase
        .get_metrics(&stack_id, &current_user.id, &range)
        .await?;
    Ok(Json(metrics))
}

pub fn resource_routes(usecase: Arc<ResourceUsecase>) -> Router {
    Router::new()
        .route("/{stack_id}/limits", get(list_stack_limits))
        .route(
            "/{stack_id}/services/{service_name}/limits",
            put(update_service_limits),
        )
        .route("/{stack_id}/metrics", get(get_stack_metrics))
        .with_state(usecase)
}
