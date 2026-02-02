use axum::{middleware as axum_middleware, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::api::middleware::auth_middleware;
use crate::api::rest::auth::protected_auth_routes;
use crate::api::rest::*;
use crate::app_state::AppState;

/// Create the main application router
pub fn create_router(state: Arc<AppState>) -> Router {
    let mut app = Router::new();

    // 1. Initial Layers (CORS, Tracing)
    let cors = create_cors_layer();

    // 2. Public Routes (Auth, Health, System)
    app = app.merge(create_public_routes(&state));

    // 3. Webhook Routes
    if let Some(webhook_router) = create_webhook_routes(&state) {
        app = app.nest("/api/webhooks", webhook_router);
    }

    // 4. Protected Routes
    app = app.nest("/api", create_protected_routes(state.clone()));

    // 5. Final Assembly & Static Files
    app = app.layer(TraceLayer::new_for_http()).layer(cors);

    add_frontend_route(app)
}

fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
}

fn create_public_routes(state: &AppState) -> Router {
    Router::new()
        .nest("/api", health_routes())
        .nest("/api/system", system_routes(state.system_usecase.clone()))
        .nest("/api/auth", auth_routes(state.auth_service.clone()))
}

fn create_webhook_routes(state: &AppState) -> Option<Router> {
    if let (Some(stack_uc), Some(log_uc)) = (&state.stack_usecase, &state.log_usecase) {
        let webhook_state = crate::api::rest::webhooks::WebhookState {
            stack_usecase: stack_uc.clone(),
            deployment_log_usecase: log_uc.clone(),
        };

        Some(
            Router::new()
                .route(
                    "/deploy/{stack_id}/{token}",
                    axum::routing::post(crate::api::rest::webhooks::trigger_deploy),
                )
                .with_state(webhook_state),
        )
    } else {
        None
    }
}

fn create_protected_routes(state: Arc<AppState>) -> Router {
    let mut routes = Router::new().merge(protected_auth_routes());

    // Add container-dependent routes if available
    if let (
        Some(stack_uc),
        Some(team_uc),
        Some(registry_uc),
        Some(env_uc),
        Some(template_uc),
        Some(resource_uc),
        Some(log_uc),
        Some(_domain_uc),
        Some(_dns_uc),
    ) = (
        &state.stack_usecase,
        &state.team_usecase,
        &state.registry_usecase,
        &state.env_usecase,
        &state.template_usecase,
        &state.resource_usecase,
        &state.log_usecase,
        &state.domain_usecase,
        &state.dns_usecase,
    ) {
        routes = routes
            .nest("/teams", team_routes(team_uc.clone()))
            .nest("/registries", registry_routes(registry_uc.clone()))
            .nest("/containers", container_routes(stack_uc.clone()))
            .nest(
                "/images",
                image_routes(
                    state.container_service.as_ref().unwrap().clone(),
                    registry_uc.clone(),
                ),
            )
            .nest("/stacks", stack_routes(stack_uc.clone()))
            .nest("/stacks", domain_routes(state.clone()))
            .nest(
                "/stacks",
                deployment_log_routes(log_uc.clone(), stack_uc.clone()),
            )
            .nest("/stacks", resource_routes(resource_uc.clone()))
            .nest(
                "/stacks",
                environment_routes(env_uc.clone(), stack_uc.clone()),
            )
            .nest("/templates", template_routes(template_uc.clone()))
            .merge(dns_routes(state.clone()));
    }

    routes.layer(axum_middleware::from_fn_with_state(
        state.auth_service.clone(),
        auth_middleware,
    ))
}

fn add_frontend_route(app: Router) -> Router {
    let frontend_dir =
        std::env::var("FRONTEND_DIR").unwrap_or_else(|_| "./frontend/build".to_string());
    if std::path::Path::new(&frontend_dir).exists() {
        tracing::info!("Serving frontend from {}", frontend_dir);
        let static_service = ServeDir::new(&frontend_dir)
            .fallback(ServeFile::new(format!("{}/index.html", frontend_dir)));
        app.fallback_service(static_service)
    } else {
        tracing::warn!(
            "Frontend directory {} not found. Dashboard will not be available.",
            frontend_dir
        );
        app
    }
}
