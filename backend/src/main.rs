mod api;
mod app_state;
mod config;
mod db;
mod domain;
mod error;
mod infrastructure;
mod services;
mod usecase;

use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::create_router;
use crate::app_state::AppState;
use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load .env file
    dotenvy::dotenv().ok();

    // 2. Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "labuh=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 3. Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("Starting Labuh server on {}", config.server_addr());

    // 4. Create database pool
    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Database connection established");

    // 5. Run migrations
    db::run_migrations(&pool).await?;
    tracing::info!("Database migrations completed");

    // 6. Initialize Application State (Dependency Injection Container)
    let state = Arc::new(AppState::new(config.clone(), pool).await?);
    tracing::info!("Application state initialized");

    // 7. Create Router
    let app = create_router(state.clone());

    // 8. Start server
    let addr = config.server_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 Labuh server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
