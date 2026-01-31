use crate::domain::models::team::{Team, TeamMember, TeamRole};
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TeamRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Team>>;
    async fn save(&self, team: &Team) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;

    // Membership
    async fn add_member(&self, team_id: &str, user_id: &str, role: TeamRole) -> Result<()>;
    async fn remove_member(&self, team_id: &str, user_id: &str) -> Result<()>;
    async fn update_member_role(&self, team_id: &str, user_id: &str, role: TeamRole) -> Result<()>;
    async fn get_members(&self, team_id: &str) -> Result<Vec<TeamMember>>;
    async fn get_user_role(&self, team_id: &str, user_id: &str) -> Result<Option<TeamRole>>;
}
