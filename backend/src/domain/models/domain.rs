use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "PascalCase")]
pub enum DomainProvider {
    Custom,
    Cloudflare,
    CPanel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "PascalCase")]
pub enum DomainType {
    Caddy,
    Tunnel,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Domain {
    pub id: String,
    pub stack_id: String,
    pub container_name: String,
    pub container_port: i32,
    pub domain: String,
    pub ssl_enabled: bool,
    pub verified: bool,
    pub provider: DomainProvider,
    #[sqlx(rename = "type")]
    pub r#type: DomainType,
    pub tunnel_id: Option<String>,
    pub dns_record_id: Option<String>,
    pub proxied: bool,
    pub show_branding: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDomain {
    pub domain: String,
    pub container_name: String,
    pub container_port: Option<i32>,
    pub provider: Option<DomainProvider>,
    pub r#type: Option<DomainType>,
    pub tunnel_id: Option<String>,
    pub tunnel_token: Option<String>,
    pub dns_record_type: Option<String>,
    pub dns_record_content: Option<String>,
    pub proxied: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DomainResponse {
    pub id: String,
    pub stack_id: String,
    pub container_name: String,
    pub container_port: i32,
    pub domain: String,
    pub ssl_enabled: bool,
    pub verified: bool,
    pub provider: DomainProvider,
    pub r#type: DomainType,
    pub tunnel_id: Option<String>,
    pub dns_record_id: Option<String>,
    pub proxied: bool,
    pub show_branding: bool,
    pub created_at: String,
}

impl From<Domain> for DomainResponse {
    fn from(d: Domain) -> Self {
        Self {
            id: d.id,
            stack_id: d.stack_id,
            container_name: d.container_name,
            container_port: d.container_port,
            domain: d.domain,
            ssl_enabled: d.ssl_enabled,
            verified: d.verified,
            provider: d.provider,
            r#type: d.r#type,
            tunnel_id: d.tunnel_id,
            dns_record_id: d.dns_record_id,
            proxied: d.proxied,
            show_branding: d.show_branding,
            created_at: d.created_at,
        }
    }
}
