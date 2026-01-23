use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub container_id: Option<String>,
    pub image: Option<String>,
    pub status: String,
    pub port: Option<i64>,
    pub env_vars: Option<String>, // JSON
    pub domains: Option<String>,  // JSON array
    pub webhook_token: Option<String>,
    pub user_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub port: Option<i64>,
    pub env_vars: Option<serde_json::Value>,
    pub domains: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProject {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub port: Option<i64>,
    pub env_vars: Option<serde_json::Value>,
    pub domains: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub container_id: Option<String>,
    pub image: Option<String>,
    pub status: String,
    pub port: Option<i64>,
    pub env_vars: Option<serde_json::Value>,
    pub domains: Option<Vec<String>>,
    pub webhook_token: Option<String>,
    pub user_id: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Project> for ProjectResponse {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            name: p.name,
            slug: p.slug,
            description: p.description,
            container_id: p.container_id,
            image: p.image,
            status: p.status,
            port: p.port,
            env_vars: p.env_vars.and_then(|s| serde_json::from_str(&s).ok()),
            domains: p.domains.and_then(|s| serde_json::from_str(&s).ok()),
            webhook_token: p.webhook_token,
            user_id: p.user_id,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

// Helper to create slug from name
pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
