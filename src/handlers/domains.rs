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
use crate::services::domain::DnsVerificationResult;

async fn list_domains(
    State(domain_service): State<Arc<DomainService>>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
) -> Result<Json<Vec<DomainResponse>>> {
    let domains = domain_service.list_domains(&stack_id).await?;
    let responses: Vec<DomainResponse> = domains.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn add_domain(
    State(domain_service): State<Arc<DomainService>>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
    Json(request): Json<CreateDomain>,
) -> Result<Json<DomainResponse>> {
    // Get stack upstream
    let upstream = domain_service.get_stack_upstream(&stack_id).await?;

    let domain = domain_service
        .add_domain(&stack_id, &request.domain, &upstream)
        .await?;
    Ok(Json(domain.into()))
}

async fn remove_domain(
    State(domain_service): State<Arc<DomainService>>,
    Extension(_current_user): Extension<CurrentUser>,
    Path((stack_id, domain)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    domain_service.remove_domain(&stack_id, &domain).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

async fn verify_domain(
    State(domain_service): State<Arc<DomainService>>,
    Extension(_current_user): Extension<CurrentUser>,
    Path((_stack_id, domain)): Path<(String, String)>,
) -> Result<Json<DnsVerificationResult>> {
    let result = domain_service.verify_domain(&domain, None).await?;
    Ok(Json(result))
}

pub fn domain_routes(domain_service: Arc<DomainService>) -> Router {
    Router::new()
        .route("/{stack_id}/domains", get(list_domains))
        .route("/{stack_id}/domains", post(add_domain))
        .route("/{stack_id}/domains/{domain}", delete(remove_domain))
        .route("/{stack_id}/domains/{domain}/verify", post(verify_domain))
        .with_state(domain_service)
}
