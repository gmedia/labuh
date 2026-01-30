use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RegistryCredential {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub registry_url: String,
    pub username: String,
    #[allow(dead_code)]
    #[serde(skip_serializing)]
    pub password_encrypted: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRegistryCredential {
    pub name: String,
    pub registry_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegistryCredentialResponse {
    pub id: String,
    pub name: String,
    pub registry_url: String,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<RegistryCredential> for RegistryCredentialResponse {
    fn from(r: RegistryCredential) -> Self {
        Self {
            id: r.id,
            name: r.name,
            registry_url: r.registry_url,
            username: r.username,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// Common registry URLs
#[allow(dead_code)]
pub mod registries {
    pub const DOCKER_HUB: &str = "docker.io";
    pub const GHCR: &str = "ghcr.io";
}
