use crate::domain::domain_repository::DomainRepository;
use crate::domain::models::Domain;
use crate::error::Result;
use async_trait::async_trait;
use sqlx::SqlitePool;

pub struct SqliteDomainRepository {
    pool: SqlitePool,
}

impl SqliteDomainRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DomainRepository for SqliteDomainRepository {
    async fn find_by_stack_id(&self, stack_id: &str) -> Result<Vec<Domain>> {
        let domains = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE stack_id = ? ORDER BY created_at DESC",
        )
        .bind(stack_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(domains)
    }

    async fn find_by_team_id(&self, team_id: &str) -> Result<Vec<Domain>> {
        let domains = sqlx::query_as::<_, Domain>(
            "SELECT d.* FROM domains d
             JOIN stacks s ON d.stack_id = s.id
             WHERE s.team_id = ?
             ORDER BY d.created_at DESC",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(domains)
    }

    async fn find_by_domain(&self, domain: &str) -> Result<Option<Domain>> {
        let domain = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE domain = ?")
            .bind(domain)
            .fetch_optional(&self.pool)
            .await?;
        Ok(domain)
    }

    async fn list_all(&self) -> Result<Vec<Domain>> {
        let domains = sqlx::query_as::<_, Domain>("SELECT * FROM domains ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        Ok(domains)
    }

    async fn create(&self, domain: Domain) -> Result<Domain> {
        sqlx::query(
            "INSERT INTO domains (id, stack_id, container_name, container_port, domain, ssl_enabled, verified, provider, type, tunnel_id, dns_record_id, proxied, show_branding, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&domain.id)
        .bind(&domain.stack_id)
        .bind(&domain.container_name)
        .bind(domain.container_port)
        .bind(&domain.domain)
        .bind(domain.ssl_enabled)
        .bind(domain.verified)
        .bind(&domain.provider)
        .bind(&domain.r#type)
        .bind(&domain.tunnel_id)
        .bind(&domain.dns_record_id)
        .bind(domain.proxied)
        .bind(domain.show_branding)
        .bind(&domain.created_at)
        .execute(&self.pool)
        .await?;

        Ok(domain)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM domains WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_verification(&self, domain: &str, verified: bool) -> Result<()> {
        sqlx::query("UPDATE domains SET verified = ? WHERE domain = ?")
            .bind(verified)
            .bind(domain)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_dns_record_id(&self, id: &str, dns_record_id: &str) -> Result<()> {
        sqlx::query("UPDATE domains SET dns_record_id = ? WHERE id = ?")
            .bind(dns_record_id)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_branding(&self, domain: &str, show_branding: bool) -> Result<()> {
        sqlx::query("UPDATE domains SET show_branding = ? WHERE domain = ?")
            .bind(show_branding)
            .bind(domain)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
