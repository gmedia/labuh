use sqlx::SqlitePool;
use std::sync::Arc;

use crate::config::Config;
use crate::domain::runtime::RuntimePort;
use crate::domain::system::SystemProvider;
use crate::infrastructure::caddy::client::CaddyClient;
use crate::infrastructure::tunnel::manager::TunnelManager;
use crate::usecase::auth::AuthUsecase;
use crate::usecase::deployment_log::DeploymentLogUsecase;
use crate::usecase::environment::EnvironmentUsecase;
use crate::usecase::metrics::MetricsUsecase;
use crate::usecase::node::NodeUsecase;
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

    // Infrastructure
    pub runtime: Arc<dyn RuntimePort>,
    pub caddy_client: Arc<CaddyClient>,
    pub system_provider: Arc<dyn SystemProvider>,
    pub tunnel_manager: Option<Arc<TunnelManager>>,

    // Usecases
    pub auth_usecase: Arc<AuthUsecase>,
    pub system_usecase: Arc<SystemUsecase>,
    pub node_usecase: Arc<NodeUsecase>,

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
    pub metrics_usecase: Option<Arc<MetricsUsecase>>,
}

impl AppState {
    pub async fn new(config: Config, pool: SqlitePool) -> anyhow::Result<Self> {
        // 1. Initialize Infrastructure

        // Runtime (Docker)
        let runtime: Arc<dyn RuntimePort> =
            Arc::new(crate::infrastructure::docker::runtime::DockerRuntimeAdapter::new().await?);

        // Caddy
        let caddy_client = Arc::new(CaddyClient::new(config.caddy_admin_api.clone()));

        // Auth Infra
        let jwt_service = Arc::new(crate::infrastructure::auth::jwt::JwtService::new(
            config.jwt_secret.clone(),
            config.jwt_expiration_hours,
        ));

        // User Repo
        let user_repo =
            Arc::new(crate::infrastructure::sqlite::user::SqliteUserRepository::new(pool.clone()));

        // 2. Initialize Core Usecases
        let auth_usecase = Arc::new(AuthUsecase::new(user_repo, jwt_service));

        let system_provider =
            Arc::new(crate::infrastructure::linux_system::LinuxSystemProvider::new());
        let system_usecase = Arc::new(crate::usecase::system::SystemUsecase::new(
            system_provider.clone(),
        ));

        let mut app_state = Self {
            _config: config.clone(),
            _pool: pool,
            runtime: runtime.clone(),
            caddy_client,
            system_provider: system_provider.clone(),
            tunnel_manager: None,
            auth_usecase,
            system_usecase,
            node_usecase: Arc::new(NodeUsecase::new(runtime.clone())),
            env_usecase: None,
            registry_usecase: None,
            stack_usecase: None,
            team_usecase: None,
            template_usecase: None,
            resource_usecase: None,
            log_usecase: None,
            domain_usecase: None,
            dns_usecase: None,
            metrics_usecase: None,
        };

        app_state.init_full_stack().await?;

        Ok(app_state)
    }

    /// Initialize full stack components (repositories, complex usecases)
    async fn init_full_stack(&mut self) -> anyhow::Result<()> {
        let pool = self._pool.clone();
        let runtime = self.runtime.clone();

        // User
        let user_repo =
            Arc::new(crate::infrastructure::sqlite::user::SqliteUserRepository::new(pool.clone()));

        // Environment
        let env_repo = Arc::new(
            crate::infrastructure::sqlite::environment::SqliteEnvironmentRepository::new(
                pool.clone(),
            ),
        );
        let env_uc = Arc::new(EnvironmentUsecase::new(env_repo));
        self.env_usecase = Some(env_uc.clone());

        // Team
        let team_repo =
            Arc::new(crate::infrastructure::sqlite::team::SqliteTeamRepository::new(pool.clone()));
        let team_uc = Arc::new(crate::usecase::team::TeamUsecase::new(
            team_repo.clone(),
            user_repo.clone(),
        ));
        self.team_usecase = Some(team_uc.clone());

        // Registry
        let registry_repo = Arc::new(
            crate::infrastructure::sqlite::registry::SqliteRegistryRepository::new(pool.clone()),
        );
        let registry_uc = Arc::new(RegistryUsecase::new(registry_repo, team_repo.clone()));
        self.registry_usecase = Some(registry_uc.clone());

        // Template
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

        // Stack & Resources
        let stack_repo = Arc::new(
            crate::infrastructure::sqlite::stack::SqliteStackRepository::new(pool.clone()),
        );

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
        let metrics_repo = Arc::new(
            crate::infrastructure::sqlite::metrics::SqliteMetricsRepository::new(pool.clone()),
        );
        let metrics_collector = Arc::new(crate::usecase::metrics_collector::MetricsCollector::new(
            stack_repo.clone(),
            resource_repo.clone(),
            metrics_repo.clone(),
            runtime.clone(),
            self.system_provider.clone(),
        ));
        tokio::spawn(async move {
            metrics_collector.start().await;
        });

        self.metrics_usecase = Some(Arc::new(MetricsUsecase::new(metrics_repo)));

        let stack_uc = Arc::new(StackUsecase::new(
            stack_repo.clone(),
            runtime.clone(),
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

        // Logs
        let log_repo = Arc::new(
            crate::infrastructure::sqlite::deployment_log::SqliteDeploymentLogRepository::new(
                pool.clone(),
            ),
        );
        self.log_usecase = Some(Arc::new(DeploymentLogUsecase::new(log_repo)));

        // Domain & DNS
        let domain_repo = Arc::new(
            crate::infrastructure::sqlite::domain::SqliteDomainRepository::new(pool.clone()),
        );
        let dns_config_repo = Arc::new(
            crate::infrastructure::sqlite::dns::SqliteDnsConfigRepository::new(pool.clone()),
        );
        let dns_uc = Arc::new(crate::usecase::dns::DnsUsecase::new(dns_config_repo));
        self.dns_usecase = Some(dns_uc.clone());

        // Initialize Tunnel Manager
        let tunnel_manager = Arc::new(TunnelManager::new(runtime.clone()));
        self.tunnel_manager = Some(tunnel_manager.clone());

        let caddy_client = self.caddy_client.clone();
        let domain_uc = Arc::new(crate::usecase::domain::DomainUsecase::new(
            domain_repo,
            stack_repo,
            caddy_client,
            dns_uc,
            runtime.clone(),
            Some(tunnel_manager),
        ));
        self.domain_usecase = Some(domain_uc.clone());

        // Prepare Infrastructure (Caddy, Networks)
        // Ensure labuh-network
        if let Err(e) = runtime.ensure_network("labuh-network").await {
            tracing::error!("Failed to create labuh-network: {}", e);
        }

        // Bootstrap Caddy
        if let Err(e) = self.caddy_client.bootstrap(&runtime).await {
            tracing::error!("Failed to bootstrap Caddy: {}", e);
        }

        // Connect Caddy to network
        if let Err(e) = runtime
            .connect_network("labuh-caddy", "labuh-network")
            .await
        {
            tracing::warn!("Could not connect Caddy to labuh-network: {}", e);
        }

        // Sync Dommains
        if let Err(e) = domain_uc.sync_all_routes().await {
            tracing::error!("Failed to sync domains to Caddy: {}", e);
        }

        Ok(())
    }
}
