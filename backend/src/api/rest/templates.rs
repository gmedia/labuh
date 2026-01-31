use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use std::sync::Arc;

use crate::domain::models::template::{Template, TemplateResponse};
use crate::error::Result;
use crate::usecase::template::TemplateUsecase;

pub fn template_routes(usecase: Arc<TemplateUsecase>) -> Router {
    Router::new()
        .route("/", get(list_templates))
        .route("/{id}", get(get_template))
        .with_state(usecase)
}

async fn list_templates(
    State(usecase): State<Arc<TemplateUsecase>>,
) -> Result<Json<Vec<TemplateResponse>>> {
    let templates = usecase.list_templates().await?;
    Ok(Json(templates))
}

async fn get_template(
    State(usecase): State<Arc<TemplateUsecase>>,
    Path(id): Path<String>,
) -> Result<Json<Template>> {
    let template = usecase.get_template(&id).await?;
    Ok(Json(template))
}
