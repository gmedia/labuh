use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait RuntimePort: Send + Sync {
    async fn pull_image(&self, image: &str, credentials: Option<(String, String)>) -> Result<()>;
    async fn create_container(&self, config: ContainerConfig) -> Result<String>;
    async fn start_container(&self, id: &str) -> Result<()>;
    async fn stop_container(&self, id: &str) -> Result<()>;
    async fn restart_container(&self, id: &str) -> Result<()>;
    async fn remove_container(&self, id: &str, force: bool) -> Result<()>;
    async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>>;
    async fn inspect_container(&self, id: &str) -> Result<ContainerInfo>;
    async fn get_logs(&self, id: &str, tail: usize) -> Result<Vec<String>>;
    async fn get_stats(&self, id: &str) -> Result<ContainerStats>;
    async fn build_image(
        &self,
        image_name: &str,
        context_path: &str,
        dockerfile_path: &str,
    ) -> Result<tokio_stream::wrappers::ReceiverStream<Result<String>>>;
    async fn exec_command(
        &self,
        id: &str,
        cmd: Vec<String>,
    ) -> Result<bollard::exec::CreateExecResults>;
    async fn connect_exec(&self, exec_id: &str) -> Result<bollard::exec::StartExecResults>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerStats {
    pub cpu_percent: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub memory_percent: f64,
    pub network_rx: u64,
    pub network_tx: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerConfig {
    pub name: String,
    pub image: String,
    pub env: Option<Vec<String>>,
    pub ports: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,
    pub labels: Option<std::collections::HashMap<String, String>>,
    pub cpu_limit: Option<f64>,
    pub memory_limit: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub state: String,
    pub status: String,
    pub labels: std::collections::HashMap<String, String>,
}
