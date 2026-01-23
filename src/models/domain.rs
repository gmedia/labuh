use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Domain {
    pub id: String,
    pub stack_id: String,
    pub domain: String,
    pub ssl_enabled: bool,
    pub verified: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDomain {
    pub domain: String,
}

#[derive(Debug, Serialize)]
pub struct DomainResponse {
    pub id: String,
    pub stack_id: String,
    pub domain: String,
    pub ssl_enabled: bool,
    pub verified: bool,
    pub created_at: String,
}

impl From<Domain> for DomainResponse {
    fn from(d: Domain) -> Self {
        Self {
            id: d.id,
            stack_id: d.stack_id,
            domain: d.domain,
            ssl_enabled: d.ssl_enabled,
            verified: d.verified,
            created_at: d.created_at,
        }
    }
}
