use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;

use crate::domain::models::team::{Team, TeamMember, TeamRole};
use crate::domain::TeamRepository;
use crate::error::Result;

pub struct SqliteTeamRepository {
    pool: SqlitePool,
}

impl SqliteTeamRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TeamRepository for SqliteTeamRepository {
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Team>> {
        let teams = sqlx::query_as::<_, Team>(
            r#"
            SELECT t.* FROM teams t
            JOIN team_members tm ON t.id = tm.team_id
            WHERE tm.user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(teams)
    }

    async fn save(&self, team: &Team) -> Result<()> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query(
            r#"
            INSERT INTO teams (id, name, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&team.id)
        .bind(&team.name)
        .bind(&team.created_at)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM teams WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn add_member(&self, team_id: &str, user_id: &str, role: TeamRole) -> Result<()> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query(
            r#"
            INSERT INTO team_members (team_id, user_id, role, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .bind(role.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn remove_member(&self, team_id: &str, user_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM team_members WHERE team_id = ? AND user_id = ?")
            .bind(team_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_member_role(&self, team_id: &str, user_id: &str, role: TeamRole) -> Result<()> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query(
            r#"
            UPDATE team_members SET role = ?, updated_at = ?
            WHERE team_id = ? AND user_id = ?
            "#,
        )
        .bind(role.to_string())
        .bind(&now)
        .bind(team_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_members(&self, team_id: &str) -> Result<Vec<TeamMember>> {
        let members = sqlx::query_as::<_, TeamMember>(
            r#"
            SELECT
                tm.team_id,
                tm.user_id,
                u.name as user_name,
                u.email as user_email,
                tm.role,
                tm.created_at,
                tm.updated_at
            FROM team_members tm
            JOIN users u ON tm.user_id = u.id
            WHERE tm.team_id = ?
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(members)
    }

    async fn get_user_role(&self, team_id: &str, user_id: &str) -> Result<Option<TeamRole>> {
        let role_str = sqlx::query_scalar::<_, String>(
            "SELECT role FROM team_members WHERE team_id = ? AND user_id = ?",
        )
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(role_str.map(TeamRole::from))
    }
}
