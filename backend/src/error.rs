use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[allow(dead_code)]
    #[error("Authentication required")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[allow(dead_code)]
    #[error("Forbidden")]
    Forbidden,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Hash error")]
    Hash,

    #[allow(dead_code)]
    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Caddy API error: {0}")]
    CaddyApi(String),

    #[error("Container runtime error: {0}")]
    ContainerRuntime(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_type, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", self.to_string()),
            AppError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                self.to_string(),
            ),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", self.to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "validation", msg.clone()),
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "Database error occurred".to_string(),
                )
            }
            AppError::Jwt(_) => (StatusCode::UNAUTHORIZED, "jwt_error", self.to_string()),
            AppError::Hash => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "hash_error",
                "Password hashing failed".to_string(),
            ),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "Internal server error".to_string(),
                )
            }
            AppError::CaddyApi(msg) => {
                tracing::error!("Caddy API error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "caddy_error",
                    msg.clone(),
                )
            }
            AppError::ContainerRuntime(msg) => {
                tracing::error!("Container runtime error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "container_error",
                    msg.clone(),
                )
            }
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, "auth_error", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.clone()),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message,
        });

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
