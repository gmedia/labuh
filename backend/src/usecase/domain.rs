use crate::domain::domain_repository::DomainRepository;
use crate::domain::models::domain::{Domain, DomainProvider, DomainType};
use crate::domain::stack_repository::StackRepository;
use crate::error::{AppError, Result};
use crate::infrastructure::caddy::client::CaddyClient;
use crate::infrastructure::tunnel::manager::TunnelManager;
use crate::usecase::dns::DnsUsecase;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct DomainUsecase {
    domain_repo: Arc<dyn DomainRepository>,
    stack_repo: Arc<dyn StackRepository>,
    caddy_client: Arc<CaddyClient>,
    dns_usecase: Arc<DnsUsecase>,
    runtime: Arc<dyn crate::domain::runtime::RuntimePort>,
    tunnel_manager: Option<Arc<TunnelManager>>,
}

pub struct AddDomainRequest {
    pub stack_id: String,
    pub domain: String,
    pub container_name: String,
    pub container_port: i32,
    pub provider: DomainProvider,
    pub domain_type: DomainType,
    pub tunnel_id: Option<String>,
    pub tunnel_token: Option<String>,
    pub dns_record_type: Option<String>,
    pub dns_record_content: Option<String>,
    pub proxied: bool,
}

impl DomainUsecase {
    pub fn new(
        domain_repo: Arc<dyn DomainRepository>,
        stack_repo: Arc<dyn StackRepository>,
        caddy_client: Arc<CaddyClient>,
        dns_usecase: Arc<DnsUsecase>,
        runtime: Arc<dyn crate::domain::runtime::RuntimePort>,
        tunnel_manager: Option<Arc<TunnelManager>>,
    ) -> Self {
        Self {
            domain_repo,
            stack_repo,
            caddy_client,
            dns_usecase,
            runtime,
            tunnel_manager,
        }
    }

    pub async fn list_team_domains(&self, team_id: &str) -> Result<Vec<Domain>> {
        self.domain_repo.find_by_team_id(team_id).await
    }

    pub async fn list_domains_by_stack(&self, stack_id: &str) -> Result<Vec<Domain>> {
        self.domain_repo.find_by_stack_id(stack_id).await
    }

    pub async fn add_domain(&self, request: AddDomainRequest) -> Result<Domain> {
        // Check if domain already exists
        if self
            .domain_repo
            .find_by_domain(&request.domain)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict(format!(
                "Domain '{}' already exists",
                request.domain
            )));
        }

        // 0. Ensure Tunnel if requested
        if let Some(token) = &request.tunnel_token
            && let Some(tm) = &self.tunnel_manager
        {
            tm.ensure_tunnel(token).await?;
        }

        // 1. Resolve tunnel ID if it's a Tunnel type domain
        let resolved_tunnel_id = if matches!(request.domain_type, DomainType::Tunnel) {
            let id = if let Some(id) = request.tunnel_id.as_deref() {
                if id.is_empty() {
                    request
                        .tunnel_token
                        .as_deref()
                        .and_then(TunnelManager::extract_tunnel_id)
                } else {
                    Some(id.to_string())
                }
            } else {
                request
                    .tunnel_token
                    .as_deref()
                    .and_then(TunnelManager::extract_tunnel_id)
            };

            if id.is_none() {
                return Err(AppError::BadRequest(
                    "Tunnel ID or Token is required for Tunnel type domain".to_string(),
                ));
            }
            id
        } else {
            None
        };

        // 2. Provision DNS record if needed
        let dns_record_id = if !matches!(request.provider, DomainProvider::Custom) {
            let (record_type, content) = if let (Some(t), Some(c)) =
                (&request.dns_record_type, &request.dns_record_content)
            {
                (t.clone(), c.clone())
            } else {
                match request.domain_type {
                    DomainType::Caddy => {
                        let ip = std::env::var("LABUH_PUBLIC_IP")
                            .unwrap_or_else(|_| "127.0.0.1".to_string());
                        ("A".to_string(), ip)
                    }
                    DomainType::Tunnel => {
                        let tunnel_id = resolved_tunnel_id.as_ref().unwrap();
                        let target = format!("{}.cfargotunnel.com", tunnel_id);
                        ("CNAME".to_string(), target)
                    }
                }
            };

            let stack = self
                .stack_repo
                .find_by_id_internal(&request.stack_id)
                .await?;
            let provider_impl = self
                .dns_usecase
                .get_provider(&stack.team_id, request.provider.clone())
                .await?;

            tracing::debug!(
                "Creating DNS record for {}: {} -> {}",
                request.domain,
                record_type,
                content
            );
            let record_id = provider_impl
                .create_record(&request.domain, &record_type, &content, request.proxied)
                .await?;

            // Setup Tunnel Ingress if type is Tunnel
            if matches!(request.domain_type, DomainType::Tunnel) {
                let tunnel_id = resolved_tunnel_id.as_ref().unwrap();
                let service_url = format!(
                    "http://{}:{}",
                    request.container_name, request.container_port
                );

                tracing::info!(
                    "Setting up tunnel ingress for {}: {} -> {}",
                    tunnel_id,
                    request.domain,
                    service_url
                );
                provider_impl
                    .setup_tunnel_ingress(tunnel_id, &request.domain, &service_url)
                    .await?;
            }
            Some(record_id)
        } else {
            None
        };

        // Create domain record
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let domain_record = Domain {
            id: id.clone(),
            stack_id: request.stack_id.to_string(),
            container_name: request.container_name.to_string(),
            container_port: request.container_port,
            domain: request.domain.to_string(),
            ssl_enabled: true,
            verified: false,
            provider: request.provider,
            r#type: request.domain_type.clone(),
            tunnel_id: resolved_tunnel_id,
            dns_record_id: dns_record_id.clone(),
            proxied: request.proxied,
            show_branding: true,
            created_at: now,
        };

        if let Err(e) = self.domain_repo.create(domain_record.clone()).await {
            // Rollback DNS if possible
            self.rollback_dns(&request.stack_id, &domain_record, dns_record_id.as_deref())
                .await;
            return Err(e);
        }

        // Add route to Caddy if it's a Caddy type domain
        if matches!(request.domain_type, DomainType::Caddy) {
            let upstream_name = self.resolve_upstream(&request.container_name).await;
            let container_upstream = format!("{}:{}", upstream_name, request.container_port);
            if let Err(e) = self
                .caddy_client
                .add_route(
                    &request.domain,
                    &container_upstream,
                    domain_record.show_branding,
                )
                .await
            {
                // Rollback DNS and DB
                self.rollback_dns(&request.stack_id, &domain_record, dns_record_id.as_deref())
                    .await;
                let _ = self.domain_repo.delete(&id).await;
                return Err(e);
            }
        }

        Ok(domain_record)
    }

    async fn rollback_dns(
        &self,
        stack_id: &str,
        domain_record: &Domain,
        dns_record_id: Option<&str>,
    ) {
        if let Some(record_id) = dns_record_id
            && let Ok(stack) = self.stack_repo.find_by_id_internal(stack_id).await
            && let Ok(provider_impl) = self
                .dns_usecase
                .get_provider(&stack.team_id, domain_record.provider.clone())
                .await
        {
            let _ = provider_impl
                .delete_record(&domain_record.domain, record_id)
                .await;
        }
    }

    pub async fn remove_domain(&self, stack_id: &str, domain: &str) -> Result<()> {
        let domain_record = self
            .domain_repo
            .find_by_domain(domain)
            .await?
            .ok_or_else(|| AppError::NotFound("Domain not found".to_string()))?;

        if domain_record.stack_id != stack_id {
            return Err(AppError::Forbidden(
                "You do not have permission to modify this domain".to_string(),
            ));
        }

        // 1. Deprovision DNS if needed
        if let Some(record_id) = &domain_record.dns_record_id {
            let stack = self.stack_repo.find_by_id_internal(stack_id).await?;
            if let Ok(provider_impl) = self
                .dns_usecase
                .get_provider(&stack.team_id, domain_record.provider.clone())
                .await
            {
                let _ = provider_impl
                    .delete_record(&domain_record.domain, record_id)
                    .await;

                // Also remove Tunnel Ingress if applicable
                if matches!(domain_record.r#type, DomainType::Tunnel)
                    && let Some(tunnel_id) = &domain_record.tunnel_id
                {
                    let _ = provider_impl
                        .remove_tunnel_ingress(tunnel_id, &domain_record.domain)
                        .await;
                }
            }
        }

        // Remove from Caddy
        if matches!(domain_record.r#type, DomainType::Caddy) {
            let _ = self.caddy_client.remove_route(&domain_record.domain).await;
        }

        // Delete from database
        self.domain_repo.delete(&domain_record.id).await
    }

    pub async fn verify_domain(&self, domain: &str) -> Result<DnsVerificationResult> {
        use hickory_resolver::TokioResolver;

        let resolver = TokioResolver::builder_tokio()
            .expect("Failed to create resolver builder")
            .build();

        let a_records = match resolver.lookup_ip(domain).await {
            Ok(lookup) => lookup
                .into_iter()
                .map(|ip: std::net::IpAddr| ip.to_string())
                .collect::<Vec<_>>(),
            Err(_) => vec![],
        };

        let cname_records = match resolver
            .lookup(domain, hickory_resolver::proto::rr::RecordType::CNAME)
            .await
        {
            Ok(lookup) => lookup
                .record_iter()
                .filter_map(|r: &hickory_resolver::proto::rr::Record| r.data().as_cname())
                .map(|cname| cname.to_string().trim_end_matches('.').to_string())
                .collect::<Vec<_>>(),
            Err(_) => vec![],
        };

        let expected_ip = std::env::var("LABUH_PUBLIC_IP").ok();
        let verified = if let Some(expected) = expected_ip {
            a_records.iter().any(|ip| ip == &expected)
        } else {
            !a_records.is_empty() || !cname_records.is_empty()
        };

        // Update database
        self.domain_repo
            .update_verification(domain, verified)
            .await?;

        Ok(DnsVerificationResult {
            domain: domain.to_string(),
            verified,
            a_records,
            cname_records,
        })
    }

    pub async fn sync_all_routes(&self) -> Result<()> {
        let domains = self.domain_repo.list_all().await?;
        for domain in domains {
            if matches!(domain.r#type, DomainType::Caddy) {
                let upstream_name = self.resolve_upstream(&domain.container_name).await;
                let container_upstream = format!("{}:{}", upstream_name, domain.container_port);
                let _ = self
                    .caddy_client
                    .add_route(&domain.domain, &container_upstream, domain.show_branding)
                    .await;
            }
        }
        Ok(())
    }

    pub async fn sync_infrastructure(&self) -> Result<()> {
        let domains = self.domain_repo.list_all().await?;
        for domain in domains {
            // 1. Sync Caddy
            if matches!(domain.r#type, DomainType::Caddy) {
                let upstream_name = self.resolve_upstream(&domain.container_name).await;
                let container_upstream = format!("{}:{}", upstream_name, domain.container_port);
                let _ = self
                    .caddy_client
                    .add_route(&domain.domain, &container_upstream, domain.show_branding)
                    .await;
            }

            // 2. Sync DNS (Create if missing)
            if !matches!(domain.provider, DomainProvider::Custom)
                && let Ok(stack) = self.stack_repo.find_by_id_internal(&domain.stack_id).await
                && let Ok(provider_impl) = self
                    .dns_usecase
                    .get_provider(&stack.team_id, domain.provider.clone())
                    .await
                && domain.dns_record_id.is_none()
            {
                // Determine record type and content
                let (record_type, content) = match domain.r#type {
                    DomainType::Caddy => {
                        let ip = std::env::var("LABUH_PUBLIC_IP")
                            .unwrap_or_else(|_| "127.0.0.1".to_string());
                        ("A".to_string(), ip)
                    }
                    DomainType::Tunnel => {
                        if let Some(tunnel_id) = domain.tunnel_id.as_deref() {
                            let target = format!("{}.cfargotunnel.com", tunnel_id);
                            ("CNAME".to_string(), target)
                        } else {
                            // Skip if tunnel_id is missing during sync
                            continue;
                        }
                    }
                };

                if let Ok(new_id) = provider_impl
                    .create_record(&domain.domain, &record_type, &content, domain.proxied)
                    .await
                {
                    let _ = self
                        .domain_repo
                        .update_dns_record_id(&domain.id, &new_id)
                        .await;
                }
            }

            // 3. Sync Tunnel Ingress
            if matches!(domain.r#type, DomainType::Tunnel)
                && let Some(tunnel_id) = &domain.tunnel_id
                && let Ok(stack) = self.stack_repo.find_by_id_internal(&domain.stack_id).await
                && let Ok(provider_impl) = self
                    .dns_usecase
                    .get_provider(&stack.team_id, domain.provider.clone())
                    .await
            {
                let service_url =
                    format!("http://{}:{}", domain.container_name, domain.container_port);
                let _ = provider_impl
                    .setup_tunnel_ingress(tunnel_id, &domain.domain, &service_url)
                    .await;
            }
        }
        Ok(())
    }

    pub async fn update_domain_dns(
        &self,
        stack_id: &str,
        domain: &str,
        record_type: &str,
        content: &str,
        proxied: bool,
    ) -> Result<()> {
        let domain_record = self
            .domain_repo
            .find_by_domain(domain)
            .await?
            .ok_or_else(|| AppError::NotFound("Domain not found".to_string()))?;

        if domain_record.stack_id != stack_id {
            return Err(AppError::Forbidden(
                "You do not have permission to modify this domain".to_string(),
            ));
        }

        if let Some(record_id) = &domain_record.dns_record_id {
            let stack = self.stack_repo.find_by_id_internal(stack_id).await?;
            let provider_impl = self
                .dns_usecase
                .get_provider(&stack.team_id, domain_record.provider.clone())
                .await?;
            provider_impl
                .update_record(domain, record_id, record_type, content, proxied)
                .await?;
        } else {
            return Err(AppError::Validation(
                "Domain does not have a managed DNS record".to_string(),
            ));
        }

        Ok(())
    }

    async fn resolve_upstream(&self, container_name: &str) -> String {
        let mut upstream_name = container_name.to_string();

        // Try to find if this container is part of a swarm service
        if let Ok(containers) = self.runtime.list_containers(true).await
            && let Some(container) = containers.iter().find(|c| {
                c.names.iter().any(|n| n.contains(container_name))
                    || c.id.starts_with(container_name)
            })
            && let Some(service_name) = container.labels.get("com.docker.swarm.service.name")
        {
            tracing::debug!(
                "Detected Swarm service '{}' for container '{}'. Using service VIP for routing.",
                service_name,
                container_name
            );
            upstream_name = service_name.clone();
        }

        upstream_name
    }

    pub async fn toggle_branding(
        &self,
        stack_id: &str,
        domain: &str,
        show_branding: bool,
    ) -> Result<()> {
        // Find domain and verify ownership
        let domain_record = self
            .domain_repo
            .find_by_domain(domain)
            .await?
            .ok_or_else(|| AppError::NotFound("Domain not found".to_string()))?;

        if domain_record.stack_id != stack_id {
            return Err(AppError::Forbidden(
                "You do not have permission to modify this domain".to_string(),
            ));
        }

        // Update database
        self.domain_repo
            .update_branding(domain, show_branding)
            .await?;

        // Re-sync Caddy route with new branding setting
        if matches!(domain_record.r#type, DomainType::Caddy) {
            let upstream_name = self.resolve_upstream(&domain_record.container_name).await;
            let container_upstream = format!("{}:{}", upstream_name, domain_record.container_port);
            self.caddy_client
                .add_route(domain, &container_upstream, show_branding)
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DnsVerificationResult {
    pub domain: String,
    pub verified: bool,
    pub a_records: Vec<String>,
    pub cname_records: Vec<String>,
}
