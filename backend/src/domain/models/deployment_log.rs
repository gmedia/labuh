use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeploymentLog {
    pub id: String,
    pub stack_id: String,
    pub trigger_type: String,
    pub status: String,
    pub logs: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateDeploymentLog {
    pub stack_id: String,
    pub trigger_type: String,
}

#[derive(Debug, Serialize)]
pub struct DeploymentLogResponse {
    pub id: String,
    pub stack_id: String,
    pub trigger_type: String,
    pub status: String,
    pub logs: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

impl From<DeploymentLog> for DeploymentLogResponse {
    fn from(log: DeploymentLog) -> Self {
        Self {
            id: log.id,
            stack_id: log.stack_id,
            trigger_type: log.trigger_type,
            status: log.status,
            logs: log.logs,
            started_at: log.started_at,
            finished_at: log.finished_at,
        }
    }
}
