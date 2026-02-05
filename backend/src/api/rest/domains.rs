use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::app_state::AppState;
use crate::domain::models::{CreateDomain, DomainProvider, DomainResponse, DomainType};
use crate::error::{AppError, Result};
use crate::usecase::domain::DnsVerificationResult;

#[derive(Deserialize)]
pub struct ListDomainsQuery {
    pub team_id: String,
}

#[derive(Deserialize)]
pub struct UpdateDnsRequest {
    pub record_type: String,
    pub content: String,
    pub proxied: bool,
}

async fn list_all_domains(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ListDomainsQuery>,
) -> Result<Json<Vec<DomainResponse>>> {
    let team_uc = state
        .team_usecase
        .as_ref()
        .ok_or(AppError::Internal("Team usecase not available".to_string()))?;
    let domain_uc = state.domain_usecase.as_ref().ok_or(AppError::Internal(
        "Domain usecase not available".to_string(),
    ))?;

    team_uc
        .verify_permission(
            &query.team_id,
            &current_user.id,
            crate::domain::models::TeamRole::Viewer,
        )
        .await?;

    let domains = domain_uc.list_team_domains(&query.team_id).await?;
    let responses: Vec<DomainResponse> = domains.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn list_domains(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
) -> Result<Json<Vec<DomainResponse>>> {
    let stack_uc = state.stack_usecase.as_ref().ok_or(AppError::Internal(
        "Stack usecase not available".to_string(),
    ))?;
    let domain_uc = state.domain_usecase.as_ref().ok_or(AppError::Internal(
        "Domain usecase not available".to_string(),
    ))?;

    stack_uc.get_stack(&stack_id, &current_user.id).await?;
    let domains = domain_uc.list_domains_by_stack(&stack_id).await?;
    let responses: Vec<DomainResponse> = domains.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn add_domain(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stack_id): Path<String>,
    Json(request): Json<CreateDomain>,
) -> Result<Json<DomainResponse>> {
    let stack_uc = state.stack_usecase.as_ref().ok_or(AppError::Internal(
        "Stack usecase not available".to_string(),
    ))?;
    let domain_uc = state.domain_usecase.as_ref().ok_or(AppError::Internal(
        "Domain usecase not available".to_string(),
    ))?;

    stack_uc.get_stack(&stack_id, &current_user.id).await?;
    let container_port = request.container_port.unwrap_or(80);
    let provider = request.provider.unwrap_or(DomainProvider::Custom);
    let domain_type = request.r#type.unwrap_or(DomainType::Caddy);

    let domain = domain_uc
        .add_domain(crate::usecase::domain::AddDomainRequest {
            stack_id: stack_id.to_string(),
            domain: request.domain.to_string(),
            container_name: request.container_name.to_string(),
            container_port,
            provider,
            domain_type,
            tunnel_id: request.tunnel_id.clone(),
            tunnel_token: request.tunnel_token.clone(),
            dns_record_type: request.dns_record_type.clone(),
            dns_record_content: request.dns_record_content.clone(),
            proxied: request.proxied.unwrap_or(false),
        })
        .await?;
    Ok(Json(domain.into()))
}

async fn remove_domain(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((stack_id, domain)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let stack_uc = state.stack_usecase.as_ref().ok_or(AppError::Internal(
        "Stack usecase not available".to_string(),
    ))?;
    let domain_uc = state.domain_usecase.as_ref().ok_or(AppError::Internal(
        "Domain usecase not available".to_string(),
    ))?;

    stack_uc.get_stack(&stack_id, &current_user.id).await?;
    domain_uc.remove_domain(&stack_id, &domain).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

async fn verify_domain(
    State(state): State<Arc<AppState>>,
    Extension(_current_user): Extension<CurrentUser>,
    Path((_stack_id, domain)): Path<(String, String)>,
) -> Result<Json<DnsVerificationResult>> {
    let domain_uc = state.domain_usecase.as_ref().ok_or(AppError::Internal(
        "Domain usecase not available".to_string(),
    ))?;
    let result = domain_uc.verify_domain(&domain).await?;
    Ok(Json(result))
}

async fn update_dns(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((stack_id, domain)): Path<(String, String)>,
    Json(request): Json<UpdateDnsRequest>,
) -> Result<Json<serde_json::Value>> {
    let stack_uc = state.stack_usecase.as_ref().ok_or(AppError::Internal(
        "Stack usecase not available".to_string(),
    ))?;
    let domain_uc = state.domain_usecase.as_ref().ok_or(AppError::Internal(
        "Domain usecase not available".to_string(),
    ))?;

    stack_uc.get_stack(&stack_id, &current_user.id).await?;
    domain_uc
        .update_domain_dns(
            &stack_id,
            &domain,
            &request.record_type,
            &request.content,
            request.proxied,
        )
        .await?;

    Ok(Json(serde_json::json!({ "status": "updated" })))
}

async fn sync_domains(
    State(state): State<Arc<AppState>>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>> {
    let domain_uc = state.domain_usecase.as_ref().ok_or(AppError::Internal(
        "Domain usecase not available".to_string(),
    ))?;

    domain_uc.sync_infrastructure().await?;

    Ok(Json(serde_json::json!({ "status": "synchronized" })))
}

#[derive(Deserialize)]
pub struct UpdateBrandingRequest {
    pub show_branding: bool,
}

async fn toggle_branding(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((stack_id, domain)): Path<(String, String)>,
    Json(request): Json<UpdateBrandingRequest>,
) -> Result<Json<serde_json::Value>> {
    let stack_uc = state.stack_usecase.as_ref().ok_or(AppError::Internal(
        "Stack usecase not available".to_string(),
    ))?;
    let domain_uc = state.domain_usecase.as_ref().ok_or(AppError::Internal(
        "Domain usecase not available".to_string(),
    ))?;

    stack_uc.get_stack(&stack_id, &current_user.id).await?;
    domain_uc
        .toggle_branding(&stack_id, &domain, request.show_branding)
        .await?;

    Ok(Json(
        serde_json::json!({ "status": "updated", "show_branding": request.show_branding }),
    ))
}

pub fn domain_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/domains", get(list_all_domains))
        .route("/{stack_id}/domains", get(list_domains))
        .route("/{stack_id}/domains", post(add_domain))
        .route("/{stack_id}/domains/{domain}", delete(remove_domain))
        .route("/{stack_id}/domains/{domain}/verify", post(verify_domain))
        .route(
            "/{stack_id}/domains/{domain}/dns",
            axum::routing::put(update_dns),
        )
        .route(
            "/{stack_id}/domains/{domain}/branding",
            axum::routing::put(toggle_branding),
        )
        .route("/domains/sync", post(sync_domains))
        .with_state(state)
}
