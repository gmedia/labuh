use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Stack {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub team_id: String,
    pub compose_content: Option<String>,
    pub status: String,
    pub webhook_token: Option<String>,
    pub cron_schedule: Option<String>,
    pub health_check_path: Option<String>,
    pub health_check_interval: i32,
    pub last_stable_images: Option<String>,
    pub git_url: Option<String>,
    pub git_branch: Option<String>,
    pub last_commit_hash: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateStack {
    pub name: String,
    pub team_id: String,
    pub compose_content: String,
}

#[derive(Debug, Serialize)]
pub struct StackResponse {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub team_id: String,
    pub compose_content: Option<String>,
    pub status: String,
    pub webhook_token: Option<String>,
    pub cron_schedule: Option<String>,
    pub health_check_path: Option<String>,
    pub health_check_interval: i32,
    pub last_stable_images: Option<String>,
    pub git_url: Option<String>,
    pub git_branch: Option<String>,
    pub last_commit_hash: Option<String>,
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
            team_id: s.team_id,
            compose_content: s.compose_content,
            status: s.status,
            webhook_token: s.webhook_token,
            cron_schedule: s.cron_schedule,
            health_check_path: s.health_check_path,
            health_check_interval: s.health_check_interval,
            last_stable_images: s.last_stable_images,
            git_url: s.git_url,
            git_branch: s.git_branch,
            last_commit_hash: s.last_commit_hash,
            container_count: 0, // Will be populated by service
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StackHealth {
    pub status: String,
    pub total: usize,
    pub running: usize,
    pub stopped: usize,
    pub unhealthy: usize,
    pub containers: Vec<ContainerHealth>,
}

#[derive(Debug, Serialize)]
pub struct ContainerHealth {
    pub id: String,
    pub name: String,
    pub state: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct StackLogEntry {
    pub container: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackBackup {
    pub name: String,
    pub compose_content: String,
    pub env_vars: Vec<BackupEnvVar>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupEnvVar {
    pub container_name: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}
