use crate::domain::models::template::{Template, TemplateEnv};
use crate::domain::TemplateRepository;
use crate::error::Result;
use async_trait::async_trait;
use sqlx::{Row, SqlitePool};

pub struct SqliteTemplateRepository {
    pool: SqlitePool,
}

impl SqliteTemplateRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TemplateRepository for SqliteTemplateRepository {
    async fn list_all(&self) -> Result<Vec<Template>> {
        let rows = sqlx::query("SELECT * FROM templates ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await?;

        let mut templates = Vec::new();
        for row in rows {
            let default_env_str: String = row.get("default_env");
            let default_env: Vec<TemplateEnv> =
                serde_json::from_str(&default_env_str).unwrap_or_default();

            templates.push(Template {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                icon: row.get("icon"),
                compose_content: row.get("compose_content"),
                default_env,
            });
        }

        Ok(templates)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Template>> {
        let row = sqlx::query("SELECT * FROM templates WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let default_env_str: String = row.get("default_env");
            let default_env: Vec<TemplateEnv> =
                serde_json::from_str(&default_env_str).unwrap_or_default();

            Ok(Some(Template {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                icon: row.get("icon"),
                compose_content: row.get("compose_content"),
                default_env,
            }))
        } else {
            Ok(None)
        }
    }

    async fn save(&self, template: &Template) -> Result<()> {
        let default_env_str =
            serde_json::to_string(&template.default_env).unwrap_or_else(|_| "[]".to_string());

        sqlx::query(
            r#"
            INSERT INTO templates (id, name, description, icon, compose_content, default_env, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                icon = excluded.icon,
                compose_content = excluded.compose_content,
                default_env = excluded.default_env,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&template.id)
        .bind(&template.name)
        .bind(&template.description)
        .bind(&template.icon)
        .bind(&template.compose_content)
        .bind(default_env_str)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM templates WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM templates")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}
