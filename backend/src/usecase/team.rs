use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::TeamRepository;
use crate::domain::models::{Team, TeamMember, TeamResponse, TeamRole};
use crate::error::{AppError, Result};

pub struct TeamUsecase {
    team_repo: Arc<dyn TeamRepository>,
    user_repo: Arc<dyn crate::domain::user_repository::UserRepository>,
}

impl TeamUsecase {
    pub fn new(
        team_repo: Arc<dyn TeamRepository>,
        user_repo: Arc<dyn crate::domain::user_repository::UserRepository>,
    ) -> Self {
        Self {
            team_repo,
            user_repo,
        }
    }

    pub async fn create_team(&self, name: &str, owner_id: &str) -> Result<Team> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let team = Team {
            id: id.clone(),
            name: name.to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        self.team_repo.save(&team).await?;
        self.team_repo
            .add_member(&id, owner_id, TeamRole::Owner)
            .await?;

        Ok(team)
    }

    pub async fn delete_team(&self, team_id: &str, actor_id: &str) -> Result<()> {
        self.verify_permission(team_id, actor_id, TeamRole::Owner)
            .await?;
        self.team_repo.delete(team_id).await
    }

    pub async fn get_user_teams(&self, user_id: &str) -> Result<Vec<TeamResponse>> {
        let teams = self.team_repo.find_by_user_id(user_id).await?;
        let mut responses = Vec::new();

        for team in teams {
            let role = self
                .team_repo
                .get_user_role(&team.id, user_id)
                .await?
                .ok_or(AppError::NotFound(
                    "Role not found for team member".to_string(),
                ))?;

            responses.push(TeamResponse { team, role });
        }

        Ok(responses)
    }

    #[allow(dead_code)]
    pub async fn add_member(
        &self,
        team_id: &str,
        user_id: &str,
        role: TeamRole,
        actor_id: &str,
    ) -> Result<()> {
        self.verify_permission(team_id, actor_id, TeamRole::Admin)
            .await?;

        self.team_repo.add_member(team_id, user_id, role).await?;
        Ok(())
    }

    pub async fn add_member_with_credentials(
        &self,
        team_id: &str,
        name: &str,
        email: &str,
        password: &str,
        role: TeamRole,
        actor_id: &str,
    ) -> Result<()> {
        self.verify_permission(team_id, actor_id, TeamRole::Admin)
            .await?;

        // 1. Check if user already exists
        let user = if let Some(existing_user) = self.user_repo.find_by_email(email).await? {
            existing_user
        } else {
            // 2. Create new user if not exists
            let id = Uuid::new_v4().to_string();
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let password_hash =
                crate::infrastructure::auth::password::PasswordService::hash_password(password)?;

            let new_user = crate::domain::models::User {
                id,
                email: email.to_string(),
                password_hash,
                name: Some(name.to_string()),
                role: "user".to_string(),
                created_at: now.clone(),
                updated_at: now,
            };
            self.user_repo.create(new_user).await?
        };

        // 3. Add to team
        self.team_repo.add_member(team_id, &user.id, role).await?;
        Ok(())
    }

    pub async fn remove_member(&self, team_id: &str, user_id: &str, actor_id: &str) -> Result<()> {
        if user_id != actor_id {
            self.verify_permission(team_id, actor_id, TeamRole::Admin)
                .await?;
        }

        // Cannot remove the owner
        let target_role = self
            .team_repo
            .get_user_role(team_id, user_id)
            .await?
            .ok_or(AppError::NotFound("Member not found".to_string()))?;

        if target_role == TeamRole::Owner {
            return Err(AppError::BadRequest(
                "Cannot remove the owner of the team".to_string(),
            ));
        }

        self.team_repo.remove_member(team_id, user_id).await?;
        Ok(())
    }

    pub async fn update_member_role(
        &self,
        team_id: &str,
        user_id: &str,
        role: TeamRole,
        actor_id: &str,
    ) -> Result<()> {
        // Cannot change your own role
        if user_id == actor_id {
            return Err(AppError::BadRequest(
                "Cannot change your own role".to_string(),
            ));
        }

        self.verify_permission(team_id, actor_id, TeamRole::Admin)
            .await?;

        // Cannot change the owner's role
        let target_role = self
            .team_repo
            .get_user_role(team_id, user_id)
            .await?
            .ok_or(AppError::NotFound("Member not found".to_string()))?;

        if target_role == TeamRole::Owner {
            return Err(AppError::BadRequest(
                "Cannot change the role of the team owner".to_string(),
            ));
        }

        // Get actor's role
        let actor_role = self
            .team_repo
            .get_user_role(team_id, actor_id)
            .await?
            .ok_or(AppError::Forbidden("Access denied".to_string()))?;

        let role_priority = |r: TeamRole| match r {
            TeamRole::Owner => 4,
            TeamRole::Admin => 3,
            TeamRole::Developer => 2,
            TeamRole::Viewer => 1,
        };

        // Cannot assign role equal or higher than your own (except Owner can do anything)
        if actor_role != TeamRole::Owner
            && role_priority(role.clone()) >= role_priority(actor_role.clone())
        {
            return Err(AppError::Forbidden(
                "Cannot assign a role equal to or higher than your own".to_string(),
            ));
        }

        // Cannot modify someone with equal or higher role (except Owner)
        if actor_role != TeamRole::Owner
            && role_priority(target_role.clone()) >= role_priority(actor_role)
        {
            return Err(AppError::Forbidden(
                "Cannot modify a member with equal or higher role".to_string(),
            ));
        }

        self.team_repo
            .update_member_role(team_id, user_id, role)
            .await
    }

    pub async fn get_members(&self, team_id: &str, actor_id: &str) -> Result<Vec<TeamMember>> {
        self.verify_permission(team_id, actor_id, TeamRole::Viewer)
            .await?;
        self.team_repo.get_members(team_id).await
    }

    pub async fn verify_permission(
        &self,
        team_id: &str,
        user_id: &str,
        required_role: TeamRole,
    ) -> Result<()> {
        let role = self
            .team_repo
            .get_user_role(team_id, user_id)
            .await?
            .ok_or(AppError::Forbidden("Access denied".to_string()))?;

        let role_priority = |r: TeamRole| match r {
            TeamRole::Owner => 4,
            TeamRole::Admin => 3,
            TeamRole::Developer => 2,
            TeamRole::Viewer => 1,
        };

        if role_priority(role) < role_priority(required_role) {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }

        Ok(())
    }
}
