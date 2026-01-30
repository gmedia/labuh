use crate::domain::models::system::SystemStats;
use crate::domain::system::SystemProvider;
use crate::error::Result;
use std::sync::Arc;

pub struct SystemUsecase {
    provider: Arc<dyn SystemProvider>,
}

impl SystemUsecase {
    pub fn new(provider: Arc<dyn SystemProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_stats(&self) -> Result<SystemStats> {
        self.provider.get_stats().await
    }
}
