use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RegistryCredential {
    pub id: String,
    pub user_id: String,
    pub team_id: String,
    pub name: String,
    pub registry_url: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_encrypted: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRegistryCredential {
    pub name: String,
    pub team_id: String,
    pub registry_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegistryCredentialResponse {
    pub id: String,
    pub user_id: String,
    pub team_id: String,
    pub name: String,
    pub registry_url: String,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<RegistryCredential> for RegistryCredentialResponse {
    fn from(c: RegistryCredential) -> Self {
        Self {
            id: c.id,
            user_id: c.user_id,
            team_id: c.team_id,
            name: c.name,
            registry_url: c.registry_url,
            username: c.username,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}
