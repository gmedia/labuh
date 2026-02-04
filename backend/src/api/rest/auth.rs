use axum::{
    Json, Router,
    extract::{Extension, State},
    routing::{get, post},
};
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::models::{AuthResponse, CreateUser, LoginRequest, UserResponse};
use crate::error::Result;
use crate::usecase::auth::AuthUsecase;

async fn register(
    State(auth_usecase): State<Arc<AuthUsecase>>,
    Json(input): Json<CreateUser>,
) -> Result<Json<AuthResponse>> {
    let response = auth_usecase.register(input).await?;
    Ok(Json(response))
}

async fn login(
    State(auth_usecase): State<Arc<AuthUsecase>>,
    Json(input): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let response = auth_usecase.login(input).await?;
    Ok(Json(response))
}

async fn setup_required(State(auth_usecase): State<Arc<AuthUsecase>>) -> Result<Json<bool>> {
    let required = auth_usecase.is_setup_required().await?;
    Ok(Json(required))
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

pub fn auth_routes(auth_usecase: Arc<AuthUsecase>) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/setup-required", get(setup_required))
        .with_state(auth_usecase)
}

pub fn protected_auth_routes() -> Router {
    Router::new().route("/me", get(me))
}
