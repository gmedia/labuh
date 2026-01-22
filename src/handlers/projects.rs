use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;

use crate::error::Result;
use crate::middleware::auth::CurrentUser;
use crate::models::{CreateProject, ProjectResponse, UpdateProject};
use crate::services::ProjectService;

async fn list_projects(
    State(project_service): State<Arc<ProjectService>>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ProjectResponse>>> {
    let projects = project_service.list_projects(&current_user.id).await?;
    let response: Vec<ProjectResponse> = projects.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

async fn get_project(
    State(project_service): State<Arc<ProjectService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<ProjectResponse>> {
    let project = project_service.get_project(&id, &current_user.id).await?;
    Ok(Json(project.into()))
}

async fn create_project(
    State(project_service): State<Arc<ProjectService>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(input): Json<CreateProject>,
) -> Result<Json<ProjectResponse>> {
    let project = project_service
        .create_project(&current_user.id, input)
        .await?;
    Ok(Json(project.into()))
}

async fn update_project(
    State(project_service): State<Arc<ProjectService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(input): Json<UpdateProject>,
) -> Result<Json<ProjectResponse>> {
    let project = project_service
        .update_project(&id, &current_user.id, input)
        .await?;
    Ok(Json(project.into()))
}

async fn delete_project(
    State(project_service): State<Arc<ProjectService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    project_service
        .delete_project(&id, &current_user.id)
        .await?;
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

pub fn project_routes(project_service: Arc<ProjectService>) -> Router {
    Router::new()
        .route("/", get(list_projects))
        .route("/", post(create_project))
        .route("/{id}", get(get_project))
        .route("/{id}", put(update_project))
        .route("/{id}", delete(delete_project))
        .with_state(project_service)
}
