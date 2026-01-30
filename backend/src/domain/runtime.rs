use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait RuntimePort: Send + Sync {
    async fn pull_image(&self, image: &str) -> Result<()>;
    async fn create_container(&self, config: ContainerConfig) -> Result<String>;
    async fn start_container(&self, id: &str) -> Result<()>;
    async fn stop_container(&self, id: &str) -> Result<()>;
    async fn remove_container(&self, id: &str, force: bool) -> Result<()>;
    async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>>;
    async fn get_logs(&self, id: &str, tail: usize) -> Result<Vec<String>>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerConfig {
    pub name: String,
    pub image: String,
    pub env: Option<Vec<String>>,
    pub ports: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,
    pub labels: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub state: String,
    pub status: String,
}
