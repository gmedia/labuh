use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DnsConfig {
    pub id: String,
    pub team_id: String,
    pub provider: String,
    pub config: String, // JSON
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    pub api_token: String,
    pub zone_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPanelConfig {
    pub host: String,
    pub token: String,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDnsConfigRequest {
    pub provider: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteDnsRecord {
    pub id: String,
    pub name: String,
    pub content: String,
    pub r#type: String,
    pub zone_id: String,
    pub zone_name: String,
}
