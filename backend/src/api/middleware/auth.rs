use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::services::auth::Claims;
use crate::services::AuthService;

#[derive(Clone)]
pub struct CurrentUser {
    pub id: String,
    pub email: String,
    pub role: String,
}

impl From<Claims> for CurrentUser {
    fn from(claims: Claims) -> Self {
        Self {
            id: claims.sub,
            email: claims.email,
            role: claims.role,
        }
    }
}

#[derive(Serialize)]
pub struct AuthError {
    error: String,
    message: String,
}

pub async fn auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<AuthError>)> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthError {
                    error: "unauthorized".to_string(),
                    message: "Missing or invalid authorization header".to_string(),
                }),
            ));
        }
    };

    match auth_service.verify_token(token) {
        Ok(claims) => {
            // Verify user still exists in database
            match auth_service.get_user_by_id(&claims.sub).await {
                Ok(_) => {
                    let current_user = CurrentUser::from(claims);
                    request.extensions_mut().insert(current_user);
                    Ok(next.run(request).await)
                }
                Err(_) => Err((
                    StatusCode::UNAUTHORIZED,
                    Json(AuthError {
                        error: "user_not_found".to_string(),
                        message: "User no longer exists. Please login again.".to_string(),
                    }),
                )),
            }
        }
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                error: "unauthorized".to_string(),
                message: "Invalid or expired token".to_string(),
            }),
        )),
    }
}
