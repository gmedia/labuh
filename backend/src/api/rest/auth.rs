use axum::{
    extract::{Extension, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::models::{AuthResponse, CreateUser, LoginRequest, UserResponse};
use crate::error::Result;
use crate::services::AuthService;

async fn register(
    State(auth_service): State<Arc<AuthService>>,
    Json(input): Json<CreateUser>,
) -> Result<Json<AuthResponse>> {
    let response = auth_service.register(input).await?;
    Ok(Json(response))
}

async fn login(
    State(auth_service): State<Arc<AuthService>>,
    Json(input): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let response = auth_service.login(input).await?;
    Ok(Json(response))
}

async fn me(Extension(current_user): Extension<CurrentUser>) -> Result<Json<UserResponse>> {
    Ok(Json(UserResponse {
        id: current_user.id,
        email: current_user.email,
        name: None,
        role: current_user.role,
        created_at: String::new(),
    }))
}

pub fn auth_routes(auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(auth_service)
}

pub fn protected_auth_routes() -> Router {
    Router::new().route("/me", get(me))
}
