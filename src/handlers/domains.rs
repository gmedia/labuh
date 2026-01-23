use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::error::Result;
use crate::middleware::auth::CurrentUser;
use crate::models::{CreateDomain, DomainResponse};
use crate::services::DomainService;

async fn list_domains(
    State(domain_service): State<Arc<DomainService>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<DomainResponse>>> {
    // TODO: Verify user owns project
    let domains = domain_service.list_domains(&project_id).await?;
    let responses: Vec<DomainResponse> = domains.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn add_domain(
    State(domain_service): State<Arc<DomainService>>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(project_id): Path<String>,
    Json(request): Json<CreateDomain>,
) -> Result<Json<DomainResponse>> {
    // Get project upstream
    let upstream = domain_service.get_project_upstream(&project_id).await?;

    let domain = domain_service
        .add_domain(&project_id, &request.domain, &upstream)
        .await?;
    Ok(Json(domain.into()))
}

async fn remove_domain(
    State(domain_service): State<Arc<DomainService>>,
    Extension(_current_user): Extension<CurrentUser>,
    Path((project_id, domain)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    domain_service.remove_domain(&project_id, &domain).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

async fn verify_domain(
    State(domain_service): State<Arc<DomainService>>,
    Extension(_current_user): Extension<CurrentUser>,
    Path((_project_id, domain)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let verified = domain_service.verify_domain(&domain).await?;
    Ok(Json(serde_json::json!({ "verified": verified })))
}

pub fn domain_routes(domain_service: Arc<DomainService>) -> Router {
    Router::new()
        .route("/{project_id}/domains", get(list_domains))
        .route("/{project_id}/domains", post(add_domain))
        .route("/{project_id}/domains/{domain}", delete(remove_domain))
        .route("/{project_id}/domains/{domain}/verify", post(verify_domain))
        .with_state(domain_service)
}
