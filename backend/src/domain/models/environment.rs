use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct StackEnvVar {
    pub id: String,
    pub stack_id: String,
    pub container_name: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StackEnvVarResponse {
    pub id: String,
    pub stack_id: String,
    pub container_name: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<StackEnvVar> for StackEnvVarResponse {
    fn from(env: StackEnvVar) -> Self {
        Self {
            id: env.id,
            stack_id: env.stack_id,
            container_name: env.container_name,
            key: env.key,
            value: if env.is_secret {
                "********".to_string()
            } else {
                env.value
            },
            is_secret: env.is_secret,
            created_at: env.created_at,
            updated_at: env.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetEnvVarRequest {
    pub container_name: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

#[derive(Debug, Deserialize)]
pub struct BulkSetEnvVarRequest {
    pub container_name: String,
    pub vars: Vec<EnvVarItem>,
}

#[derive(Debug, Deserialize)]
pub struct EnvVarItem {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}
