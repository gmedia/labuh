use sqlx::SqlitePool;
use std::sync::Arc;

use crate::config::Config;
use crate::services::{AuthService, CaddyService, ContainerService, NetworkService};
use crate::usecase::deployment_log::DeploymentLogUsecase;
use crate::usecase::environment::EnvironmentUsecase;
use crate::usecase::registry::RegistryUsecase;
use crate::usecase::resource::ResourceUsecase;
use crate::usecase::stack::StackUsecase;
use crate::usecase::system::SystemUsecase;
use crate::usecase::team::TeamUsecase;
use crate::usecase::template::TemplateUsecase;

/// Central application state (Dependency Injection Container)
pub struct AppState {
    pub _config: Config,
    pub _pool: SqlitePool,
    pub auth_service: Arc<AuthService>,
    pub container_service: Option<Arc<ContainerService>>,
    pub _caddy_service: Arc<CaddyService>,
    pub tunnel_service: Option<Arc<crate::services::tunnel::TunnelService>>,
    pub system_usecase: Arc<SystemUsecase>,

    // Optional/Conditional Usecases
    pub env_usecase: Option<Arc<EnvironmentUsecase>>,
    pub registry_usecase: Option<Arc<RegistryUsecase>>,
    pub stack_usecase: Option<Arc<StackUsecase>>,
    pub team_usecase: Option<Arc<TeamUsecase>>,
    pub template_usecase: Option<Arc<TemplateUsecase>>,
    pub resource_usecase: Option<Arc<ResourceUsecase>>,
    pub log_usecase: Option<Arc<DeploymentLogUsecase>>,
    pub domain_usecase: Option<Arc<crate::usecase::domain::DomainUsecase>>,
    pub dns_usecase: Option<Arc<crate::usecase::dns::DnsUsecase>>,
}

impl AppState {
    pub async fn new(config: Config, pool: SqlitePool) -> anyhow::Result<Self> {
        // 1. Core Services
        let (auth_service, container_service, caddy_service, system_usecase) =
            Self::init_core_services(&config, &pool).await;

        // 2. Initialize Infrastructure & Usecases
        let mut app_state = Self {
            _config: config,
            _pool: pool,
            auth_service,
            container_service,
            _caddy_service: caddy_service,
            tunnel_service: None,
            system_usecase,
            env_usecase: None,
            registry_usecase: None,
            stack_usecase: None,
            team_usecase: None,
            template_usecase: None,
            resource_usecase: None,
            log_usecase: None,
            domain_usecase: None,
            dns_usecase: None,
        };

        if let Some(container_svc) = app_state.container_service.clone() {
            app_state.init_usecases().await?;
            app_state.init_infrastructure(&container_svc).await?;
        }

        Ok(app_state)
    }

    /// Initialize core services that always run
    async fn init_core_services(
        config: &Config,
        pool: &SqlitePool,
    ) -> (
        Arc<AuthService>,
        Option<Arc<ContainerService>>,
        Arc<CaddyService>,
        Arc<SystemUsecase>,
    ) {
        let auth_service = Arc::new(AuthService::new(
            pool.clone(),
            config.jwt_secret.clone(),
            config.jwt_expiration_hours,
        ));

        let container_service = match ContainerService::new().await {
            Ok(service) => {
                tracing::info!("Container runtime connected");
                Some(Arc::new(service))
            }
            Err(e) => {
                tracing::warn!(
                    "Container runtime not available: {}. Container features disabled.",
                    e
                );
                None
            }
        };

        let caddy_service = Arc::new(CaddyService::new(config.caddy_admin_api.clone()));

        let system_provider =
            Arc::new(crate::infrastructure::linux_system::LinuxSystemProvider::new());
        let system_usecase = Arc::new(crate::usecase::system::SystemUsecase::new(system_provider));

        (
            auth_service,
            container_service,
            caddy_service,
            system_usecase,
        )
    }

    /// Bootstrap Infrastructure (Network, Caddy, Domains, Tunnels)
    async fn init_infrastructure(
        &mut self,
        container_svc: &Arc<ContainerService>,
    ) -> anyhow::Result<()> {
        let network_service = Arc::new(NetworkService::new(container_svc.clone()));
        if let Err(e) = network_service.ensure_labuh_network().await {
            tracing::error!("Failed to create labuh-network: {}", e);
        }

        self.tunnel_service = Some(Arc::new(crate::services::tunnel::TunnelService::new(
            container_svc.clone(),
            network_service.clone(),
        )));

        if let Err(e) = self._caddy_service.bootstrap(container_svc).await {
            tracing::error!("Failed to bootstrap Caddy: {}", e);
        }

        if let Err(e) = network_service.connect_container("labuh-caddy").await {
            tracing::warn!("Could not connect Caddy to labuh-network: {}", e);
        }

        if let Some(ref domain_uc) = self.domain_usecase {
            if let Err(e) = domain_uc.sync_all_routes().await {
                tracing::error!("Failed to sync domains to Caddy: {}", e);
            }
        }

        Ok(())
    }

    /// Initialize all repositories and usecases
    async fn init_usecases(&mut self) -> anyhow::Result<()> {
        let pool = self._pool.clone();

        let env_repo = Arc::new(
            crate::infrastructure::sqlite::environment::SqliteEnvironmentRepository::new(
                pool.clone(),
            ),
        );
        let env_uc = Arc::new(EnvironmentUsecase::new(env_repo));
        self.env_usecase = Some(env_uc.clone());

        let team_repo =
            Arc::new(crate::infrastructure::sqlite::team::SqliteTeamRepository::new(pool.clone()));
        let team_uc = Arc::new(TeamUsecase::new(team_repo.clone()));
        self.team_usecase = Some(team_uc.clone());

        let registry_repo = Arc::new(
            crate::infrastructure::sqlite::registry::SqliteRegistryRepository::new(pool.clone()),
        );
        let registry_uc = Arc::new(RegistryUsecase::new(registry_repo, team_repo.clone()));
        self.registry_usecase = Some(registry_uc.clone());

        let template_repo = Arc::new(
            crate::infrastructure::sqlite::template::SqliteTemplateRepository::new(pool.clone()),
        );
        let template_uc = Arc::new(TemplateUsecase::new(template_repo));
        self.template_usecase = Some(template_uc.clone());

        // Background Task: Seed Templates
        let t_uc = template_uc.clone();
        tokio::spawn(async move {
            if let Err(e) = t_uc.seed_default_templates().await {
                tracing::error!("Failed to seed default templates: {}", e);
            }
        });

        let stack_repo = Arc::new(
            crate::infrastructure::sqlite::stack::SqliteStackRepository::new(pool.clone()),
        );
        let runtime_adapter =
            Arc::new(crate::infrastructure::docker::runtime::DockerRuntimeAdapter::new().await?);

        let resource_repo = Arc::new(
            crate::infrastructure::sqlite::resource::SqliteResourceRepository::new(pool.clone()),
        );
        let resource_uc = Arc::new(ResourceUsecase::new(
            resource_repo.clone(),
            stack_repo.clone(),
            team_repo.clone(),
        ));
        self.resource_usecase = Some(resource_uc);

        // Background Task: Metrics Collector
        let metrics_collector = Arc::new(crate::usecase::metrics_collector::MetricsCollector::new(
            stack_repo.clone(),
            resource_repo.clone(),
            runtime_adapter.clone(),
        ));
        tokio::spawn(async move {
            metrics_collector.start().await;
        });

        let stack_uc = Arc::new(StackUsecase::new(
            stack_repo.clone(),
            runtime_adapter.clone(),
            env_uc,
            registry_uc,
            resource_repo.clone(),
            team_repo.clone(),
        ));
        self.stack_usecase = Some(stack_uc.clone());

        // Background Task: Scheduler
        let st_uc = stack_uc.clone();
        let st_repo = stack_repo.clone();
        tokio::spawn(async move {
            let scheduler = Arc::new(crate::usecase::scheduler::AutomationScheduler::new(
                st_uc, st_repo,
            ));
            scheduler.start().await;
        });

        let log_repo = Arc::new(
            crate::infrastructure::sqlite::deployment_log::SqliteDeploymentLogRepository::new(
                pool.clone(),
            ),
        );
        self.log_usecase = Some(Arc::new(DeploymentLogUsecase::new(log_repo)));

        let domain_repo = Arc::new(
            crate::infrastructure::sqlite::domain::SqliteDomainRepository::new(pool.clone()),
        );
        let dns_config_repo = Arc::new(
            crate::infrastructure::sqlite::dns::SqliteDnsConfigRepository::new(pool.clone()),
        );

        let dns_uc = Arc::new(crate::usecase::dns::DnsUsecase::new(dns_config_repo));
        self.dns_usecase = Some(dns_uc.clone());

        let caddy_svc = self._caddy_service.clone();
        let domain_uc = Arc::new(crate::usecase::domain::DomainUsecase::new(
            domain_repo,
            stack_repo,
            caddy_svc,
            dns_uc,
        ));
        self.domain_usecase = Some(domain_uc);

        Ok(())
    }
}
