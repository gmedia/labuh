use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Stack {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub compose_content: Option<String>,
    pub status: String,
    pub webhook_token: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateStack {
    pub name: String,
    pub compose_content: String,
}

#[derive(Debug, Serialize)]
pub struct StackResponse {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub compose_content: Option<String>,
    pub status: String,
    pub webhook_token: Option<String>,
    pub container_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Stack> for StackResponse {
    fn from(s: Stack) -> Self {
        Self {
            id: s.id,
            name: s.name,
            user_id: s.user_id,
            compose_content: s.compose_content,
            status: s.status,
            webhook_token: s.webhook_token,
            container_count: 0, // Will be populated by service
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}
