use crate::domain::models::Stack;
use crate::domain::stack_repository::StackRepository;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;

pub struct SqliteStackRepository {
    pool: SqlitePool,
}

impl SqliteStackRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StackRepository for SqliteStackRepository {
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Stack>> {
        let stacks = sqlx::query_as::<_, Stack>(
            "SELECT * FROM stacks WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(stacks)
    }

    async fn find_by_id(&self, id: &str, user_id: &str) -> Result<Stack> {
        let stack = sqlx::query_as::<_, Stack>("SELECT * FROM stacks WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Stack not found".to_string()))?;

        Ok(stack)
    }

    async fn find_by_id_internal(&self, id: &str) -> Result<Stack> {
        let stack = sqlx::query_as::<_, Stack>("SELECT * FROM stacks WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Stack not found".to_string()))?;

        Ok(stack)
    }

    async fn create(&self, stack: Stack) -> Result<Stack> {
        sqlx::query(
            "INSERT INTO stacks (id, name, user_id, compose_content, status, webhook_token, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&stack.id)
        .bind(&stack.name)
        .bind(&stack.user_id)
        .bind(&stack.compose_content)
        .bind(&stack.status)
        .bind(&stack.webhook_token)
        .bind(&stack.created_at)
        .bind(&stack.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(stack)
    }

    async fn update_status(&self, id: &str, status: &str) -> Result<()> {
        sqlx::query("UPDATE stacks SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn update_compose(&self, id: &str, content: &str) -> Result<()> {
        sqlx::query("UPDATE stacks SET compose_content = ?, updated_at = ? WHERE id = ?")
            .bind(content)
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn update_webhook_token(&self, id: &str, token: &str) -> Result<()> {
        sqlx::query("UPDATE stacks SET webhook_token = ?, updated_at = ? WHERE id = ?")
            .bind(token)
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM stacks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn validate_webhook_token(&self, id: &str, token: &str) -> Result<Stack> {
        let stack = self.find_by_id_internal(id).await?;

        match &stack.webhook_token {
            Some(t) if t == token => Ok(stack),
            _ => Err(AppError::Auth("Invalid webhook token".to_string())),
        }
    }
}
