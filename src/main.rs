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
    auth_routes, container_routes, deploy_routes, health_routes, image_routes, project_routes,
    streaming_routes, system_routes,
};
use crate::middleware::auth_middleware;
use crate::services::{AuthService, CaddyService, ContainerService, DeployService, ProjectService};

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

    // Create project service
    let project_service = Arc::new(ProjectService::new(pool.clone()));

    // Create Caddy service
    let caddy_service = Arc::new(CaddyService::new(config.caddy_admin_api.clone()));

    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Protected routes (require authentication)
    let mut protected_routes = Router::new()
        .merge(protected_auth_routes())
        .nest("/projects", project_routes(project_service.clone()));

    // Add container and deploy routes if container runtime is available
    if let Some(ref container_svc) = container_service {
        // Create deploy service
        let deploy_service = Arc::new(DeployService::new(
            container_svc.clone(),
            project_service.clone(),
            caddy_service.clone(),
            std::env::var("BASE_DOMAIN").unwrap_or_else(|_| "localhost".to_string()),
        ));

        protected_routes = protected_routes
            .nest("/containers", container_routes(container_svc.clone()))
            .nest("/containers", streaming_routes(container_svc.clone()))
            .nest("/images", image_routes(container_svc.clone()))
            .nest("/projects", deploy_routes(deploy_service));
    }

    let protected_routes = protected_routes.layer(axum_middleware::from_fn_with_state(
        auth_service.clone(),
        auth_middleware,
    ));

    // Build application router
    let app = Router::new()
        .nest("/api", health_routes())
        .nest("/api/system", system_routes())
        .nest("/api/auth", auth_routes(auth_service.clone()))
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
