use crate::domain::dns_repository::DnsConfigRepository;
use crate::domain::models::dns::DnsConfig;
use crate::error::Result;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;

pub struct SqliteDnsConfigRepository {
    pool: SqlitePool,
}

impl SqliteDnsConfigRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DnsConfigRepository for SqliteDnsConfigRepository {
    async fn find_by_team_id(&self, team_id: &str) -> Result<Vec<DnsConfig>> {
        let configs = sqlx::query_as::<_, DnsConfig>("SELECT * FROM dns_configs WHERE team_id = ?")
            .bind(team_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(configs)
    }

    async fn find_by_team_and_provider(
        &self,
        team_id: &str,
        provider: &str,
    ) -> Result<Option<DnsConfig>> {
        let config = sqlx::query_as::<_, DnsConfig>(
            "SELECT * FROM dns_configs WHERE team_id = ? AND provider = ?",
        )
        .bind(team_id)
        .bind(provider)
        .fetch_optional(&self.pool)
        .await?;
        Ok(config)
    }

    async fn save(&self, config: DnsConfig) -> Result<DnsConfig> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO dns_configs (id, team_id, provider, config, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(team_id, provider) DO UPDATE SET
             config = excluded.config,
             updated_at = excluded.updated_at",
        )
        .bind(&config.id)
        .bind(&config.team_id)
        .bind(&config.provider)
        .bind(&config.config)
        .bind(&config.created_at)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(config)
    }

    async fn delete(&self, team_id: &str, provider: &str) -> Result<()> {
        sqlx::query("DELETE FROM dns_configs WHERE team_id = ? AND provider = ?")
            .bind(team_id)
            .bind(provider)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
