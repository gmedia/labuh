use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::{slugify, CreateProject, Project, UpdateProject};

pub struct ProjectService {
    pool: SqlitePool,
}

impl ProjectService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_projects(&self, user_id: &str) -> Result<Vec<Project>> {
        let projects = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(projects)
    }

    pub async fn get_project(&self, id: &str, user_id: &str) -> Result<Project> {
        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::NotFound("Project not found".to_string()))
    }

    pub async fn get_project_by_slug(&self, slug: &str, user_id: &str) -> Result<Project> {
        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE slug = ? AND user_id = ?")
            .bind(slug)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::NotFound("Project not found".to_string()))
    }

    fn generate_token() -> String {
        use rand::{distributions::Alphanumeric, Rng};
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    pub async fn create_project(&self, user_id: &str, input: CreateProject) -> Result<Project> {
        let id = Uuid::new_v4().to_string();
        let slug = slugify(&input.name);
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let webhook_token = Self::generate_token();

        // Check if slug already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM projects WHERE slug = ? AND user_id = ?",
        )
        .bind(&slug)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        if existing > 0 {
            return Err(AppError::Conflict(format!(
                "Project with slug '{}' already exists",
                slug
            )));
        }

        let env_vars = input.env_vars.map(|v| v.to_string());
        let domains = input
            .domains
            .map(|v| serde_json::to_string(&v).unwrap_or_default());

        sqlx::query(
            r#"
            INSERT INTO projects (id, name, slug, description, image, port, env_vars, domains, webhook_token, user_id, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'stopped', ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&slug)
        .bind(&input.description)
        .bind(&input.image)
        .bind(input.port)
        .bind(&env_vars)
        .bind(&domains)
        .bind(webhook_token)
        .bind(user_id)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        self.get_project(&id, user_id).await
    }

    pub async fn update_project(
        &self,
        id: &str,
        user_id: &str,
        input: UpdateProject,
    ) -> Result<Project> {
        let existing = self.get_project(id, user_id).await?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let name = input.name.unwrap_or(existing.name);
        let description = input.description.or(existing.description);
        let image = input.image.or(existing.image);
        let port = input.port.or(existing.port);
        let env_vars = input.env_vars.map(|v| v.to_string()).or(existing.env_vars);
        let domains = input
            .domains
            .map(|v| serde_json::to_string(&v).unwrap_or_default())
            .or(existing.domains);

        sqlx::query(
            r#"
            UPDATE projects
            SET name = ?, description = ?, image = ?, port = ?, env_vars = ?, domains = ?, updated_at = ?
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(&name)
        .bind(&description)
        .bind(&image)
        .bind(port)
        .bind(&env_vars)
        .bind(&domains)
        .bind(&now)
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        self.get_project(id, user_id).await
    }

    pub async fn delete_project(&self, id: &str, user_id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM projects WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Project not found".to_string()));
        }

        Ok(())
    }

    pub async fn update_project_status(
        &self,
        id: &str,
        status: &str,
        container_id: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        sqlx::query(
            "UPDATE projects SET status = ?, container_id = ?, updated_at = ? WHERE id = ?",
        )
        .bind(status)
        .bind(container_id)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn regenerate_webhook_token(&self, id: &str, user_id: &str) -> Result<Project> {
        let _existing = self.get_project(id, user_id).await?;
        let token = Self::generate_token();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        sqlx::query("UPDATE projects SET webhook_token = ?, updated_at = ? WHERE id = ?")
            .bind(&token)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        self.get_project(id, user_id).await
    }

    pub async fn validate_webhook_token(&self, id: &str, token: &str) -> Result<Project> {
        let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        if let Some(stored_token) = &project.webhook_token {
            if stored_token == token {
                return Ok(project);
            }
        }

        Err(AppError::Unauthorized)
    }
}
