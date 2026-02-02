use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::app_state::AppState;
use crate::domain::models::dns::{CreateDnsConfigRequest, DnsConfig};
use crate::domain::models::TeamRole;
use crate::error::{AppError, Result};

async fn list_dns_configs(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(team_id): Path<String>,
) -> Result<Json<Vec<DnsConfig>>> {
    let team_uc = state
        .team_usecase
        .as_ref()
        .ok_or(AppError::Internal("Team usecase not available".to_string()))?;
    let dns_uc = state
        .dns_usecase
        .as_ref()
        .ok_or(AppError::Internal("DNS usecase not available".to_string()))?;

    team_uc
        .verify_permission(&team_id, &current_user.id, TeamRole::Admin)
        .await?;
    let configs = dns_uc.list_configs(&team_id).await?;
    Ok(Json(configs))
}

async fn save_dns_config(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(team_id): Path<String>,
    Json(request): Json<CreateDnsConfigRequest>,
) -> Result<Json<DnsConfig>> {
    let team_uc = state
        .team_usecase
        .as_ref()
        .ok_or(AppError::Internal("Team usecase not available".to_string()))?;
    let dns_uc = state
        .dns_usecase
        .as_ref()
        .ok_or(AppError::Internal("DNS usecase not available".to_string()))?;

    team_uc
        .verify_permission(&team_id, &current_user.id, TeamRole::Admin)
        .await?;
    let config = dns_uc
        .save_config(&team_id, &request.provider, request.config)
        .await?;
    Ok(Json(config))
}

async fn remove_dns_config(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((team_id, provider)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let team_uc = state
        .team_usecase
        .as_ref()
        .ok_or(AppError::Internal("Team usecase not available".to_string()))?;
    let dns_uc = state
        .dns_usecase
        .as_ref()
        .ok_or(AppError::Internal("DNS usecase not available".to_string()))?;

    team_uc
        .verify_permission(&team_id, &current_user.id, TeamRole::Admin)
        .await?;
    dns_uc.remove_config(&team_id, &provider).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

async fn get_available_domains(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((team_id, provider)): Path<(String, String)>,
) -> Result<Json<Vec<String>>> {
    let team_uc = state
        .team_usecase
        .as_ref()
        .ok_or(AppError::Internal("Team usecase not available".to_string()))?;
    let dns_uc = state
        .dns_usecase
        .as_ref()
        .ok_or(AppError::Internal("DNS usecase not available".to_string()))?;

    team_uc
        .verify_permission(&team_id, &current_user.id, TeamRole::Viewer)
        .await?;

    let provider_enum = match provider.to_lowercase().as_str() {
        "cloudflare" => crate::domain::models::DomainProvider::Cloudflare,
        "cpanel" => crate::domain::models::DomainProvider::CPanel,
        _ => {
            return Err(AppError::BadRequest(format!(
                "Unsupported provider: {}",
                provider
            )))
        }
    };

    let domains = dns_uc
        .list_available_domains(&team_id, provider_enum)
        .await?;
    Ok(Json(domains))
}

async fn get_remote_records(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((team_id, provider)): Path<(String, String)>,
) -> Result<Json<Vec<crate::domain::models::dns::RemoteDnsRecord>>> {
    let team_uc = state
        .team_usecase
        .as_ref()
        .ok_or(AppError::Internal("Team usecase not available".to_string()))?;
    let dns_uc = state
        .dns_usecase
        .as_ref()
        .ok_or(AppError::Internal("DNS usecase not available".to_string()))?;

    team_uc
        .verify_permission(&team_id, &current_user.id, TeamRole::Viewer)
        .await?;

    let provider_enum = match provider.to_lowercase().as_str() {
        "cloudflare" => crate::domain::models::DomainProvider::Cloudflare,
        "cpanel" => crate::domain::models::DomainProvider::CPanel,
        _ => {
            return Err(AppError::BadRequest(format!(
                "Unsupported provider: {}",
                provider
            )))
        }
    };

    let records = dns_uc.list_remote_records(&team_id, provider_enum).await?;
    Ok(Json(records))
}

pub fn dns_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/teams/{team_id}/dns-configs", get(list_dns_configs))
        .route("/teams/{team_id}/dns-configs", post(save_dns_config))
        .route(
            "/teams/{team_id}/dns-configs/{provider}",
            delete(remove_dns_config),
        )
        .route(
            "/teams/{team_id}/dns-configs/{provider}/available-domains",
            get(get_available_domains),
        )
        .route(
            "/teams/{team_id}/dns-configs/{provider}/remote-records",
            get(get_remote_records),
        )
        .with_state(state)
}
