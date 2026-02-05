use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, put},
};
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::models::{CreateTeamRequest, TeamMember, TeamResponse, TeamRole};
use crate::error::Result;
use crate::usecase::team::TeamUsecase;
use axum::Extension;

#[derive(serde::Deserialize)]
pub struct AddMemberRequest {
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub role: String,
}

pub fn team_routes(usecase: Arc<TeamUsecase>) -> Router {
    Router::new()
        .route("/", get(list_teams).post(create_team))
        .route("/{id}", delete(remove_team))
        .route("/{id}/members", get(get_members).post(add_member))
        .route("/{id}/members/{user_id}", delete(remove_member))
        .route("/{id}/members/{user_id}/role", put(update_member_role))
        .with_state(usecase)
}

async fn list_teams(
    State(usecase): State<Arc<TeamUsecase>>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<TeamResponse>>> {
    let teams = usecase.get_user_teams(&user.id).await?;
    Ok(Json(teams))
}

async fn create_team(
    State(usecase): State<Arc<TeamUsecase>>,
    Extension(user): Extension<CurrentUser>,
    Json(payload): Json<CreateTeamRequest>,
) -> Result<Json<TeamResponse>> {
    let team = usecase.create_team(&payload.name, &user.id).await?;
    Ok(Json(TeamResponse {
        team,
        role: TeamRole::Owner,
    }))
}

async fn remove_team(
    State(usecase): State<Arc<TeamUsecase>>,
    Path(id): Path<String>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<()>> {
    usecase.delete_team(&id, &user.id).await?;
    Ok(Json(()))
}

async fn get_members(
    State(usecase): State<Arc<TeamUsecase>>,
    Path(id): Path<String>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<TeamMember>>> {
    let members = usecase.get_members(&id, &user.id).await?;
    Ok(Json(members))
}

async fn add_member(
    State(usecase): State<Arc<TeamUsecase>>,
    Path(id): Path<String>,
    Extension(user): Extension<CurrentUser>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<Json<()>> {
    if let Some(password) = payload.password {
        // Create new user and add to team
        usecase
            .add_member_with_credentials(
                &id,
                &payload.name,
                &payload.email,
                &password,
                TeamRole::from(payload.role),
                &user.id,
            )
            .await?;
    } else {
        // Try to find existing user by email
        // We'll actually need a way to look up user_id by email if we want to support existing users without passwords here.
        // For now, let's assume we always provide name/email/password for new flow.
        return Err(crate::error::AppError::BadRequest(
            "Password is required for new members".to_string(),
        ));
    }
    Ok(Json(()))
}

async fn remove_member(
    State(usecase): State<Arc<TeamUsecase>>,
    Path((id, target_user_id)): Path<(String, String)>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<()>> {
    usecase
        .remove_member(&id, &target_user_id, &user.id)
        .await?;
    Ok(Json(()))
}

#[derive(serde::Deserialize)]
struct UpdateMemberRoleRequest {
    role: TeamRole,
}

async fn update_member_role(
    State(usecase): State<Arc<TeamUsecase>>,
    Path((id, target_user_id)): Path<(String, String)>,
    Extension(user): Extension<CurrentUser>,
    Json(payload): Json<UpdateMemberRoleRequest>,
) -> Result<Json<()>> {
    usecase
        .update_member_role(&id, &target_user_id, payload.role, &user.id)
        .await?;
    Ok(Json(()))
}
