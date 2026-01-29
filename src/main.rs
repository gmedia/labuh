mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod services;

use std::sync::Arc;

use axum::{middleware as axum_middleware, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::handlers::auth::protected_auth_routes;
use crate::handlers::{
    auth_routes, container_routes, deployment_log_routes, domain_routes,
    environment_routes, health_routes, image_routes,
    registry_routes, stack_routes, system_routes,
};
use crate::middleware::auth_middleware;
use crate::services::{
    AuthService, CaddyService, ContainerService, DeploymentLogService, DomainService,
    RegistryService, StackService,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "labuh=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("Starting Labuh server on {}", config.server_addr());

    // Create database pool
    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Database connection established");

    // Run migrations
    db::run_migrations(&pool).await?;
    tracing::info!("Database migrations completed");

    // Create services
    let auth_service = Arc::new(AuthService::new(
        pool.clone(),
        config.jwt_secret.clone(),
        config.jwt_expiration_hours,
    ));

    // Create container service (optional - may fail if Docker is not available)
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

    // Create Caddy service
    let caddy_service = Arc::new(CaddyService::new(config.caddy_admin_api.clone()));

    // Bootstrap Caddy and Network if container service is available
    if let Some(ref container_svc) = container_service {
        // Create and bootstrap network service
        let network_service = crate::services::NetworkService::new(container_svc.clone());
        tracing::info!("Ensuring labuh-network exists...");
        if let Err(e) = network_service.ensure_labuh_network().await {
            tracing::error!("Failed to create labuh-network: {}", e);
        }

        // Bootstrap Caddy
        tracing::info!("Bootstrapping Caddy...");
        if let Err(e) = caddy_service.bootstrap(container_svc).await {
            tracing::error!("Failed to bootstrap Caddy: {}. Ensure port 80/443 are free.", e);
        }

        // Connect Caddy to labuh-network
        if let Err(e) = network_service.connect_container("labuh-caddy").await {
            tracing::warn!("Could not connect Caddy to labuh-network: {}", e);
        }

        // Sync all domains to Caddy after bootstrap
        let ds = DomainService::new(pool.clone(), caddy_service.clone());
        tracing::info!("Syncing domains to Caddy...");
        if let Err(e) = ds.sync_all_routes().await {
            tracing::error!("Failed to sync domains to Caddy: {}", e);
        }
    }

    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ]);

    // Create domain service
    let domain_service = Arc::new(DomainService::new(pool.clone(), caddy_service.clone()));

    // Create registry service
    let registry_service = Arc::new(RegistryService::new(pool.clone()));

    // Create deployment log service
    let deployment_log_service = Arc::new(DeploymentLogService::new(pool.clone()));

    // Protected routes (require authentication)
    let mut protected_routes = Router::new()
        .merge(protected_auth_routes())
        .nest("/registries", registry_routes(registry_service));

    // Webhook routes (only available if container service is available)
    let mut webhook_routes: Option<Router> = None;

    // Add container, stack, and deploy routes if container runtime is available
    if let Some(ref container_svc) = container_service {
        // Create environment service
        let env_service = Arc::new(crate::services::EnvironmentService::new(pool.clone()));

        // Create stack service
        let stack_service = Arc::new(StackService::new(pool.clone(), container_svc.clone(), env_service.clone()));

        protected_routes = protected_routes
            .nest("/containers", container_routes(container_svc.clone()))
            .nest("/images", image_routes(container_svc.clone()))
            .nest("/stacks", stack_routes(stack_service.clone()))
            .nest("/stacks", domain_routes(domain_service))
            .nest("/stacks", deployment_log_routes(deployment_log_service.clone(), stack_service.clone()))
            .nest("/stacks", environment_routes(env_service, stack_service.clone()));

        // Create webhook state and routes
        let webhook_state = handlers::webhooks::WebhookState {
            stack_service: stack_service.clone(),
            deployment_log_service: deployment_log_service.clone(),
        };

        webhook_routes = Some(
            Router::new()
                .route("/deploy/{stack_id}/{token}", axum::routing::post(handlers::webhooks::trigger_deploy))
                .with_state(webhook_state)
        );
    }

    let protected_routes = protected_routes.layer(axum_middleware::from_fn_with_state(
        auth_service.clone(),
        auth_middleware,
    ));

    // Build application router
    let mut app = Router::new()
        .nest("/api", health_routes())
        .nest("/api/system", system_routes())
        .nest("/api/auth", auth_routes(auth_service.clone()));

    if let Some(webhook_router) = webhook_routes {
        app = app.nest("/api/webhooks", webhook_router);
    }

    let app = app
        .nest("/api", protected_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Start server
    let addr = config.server_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 Labuh server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
