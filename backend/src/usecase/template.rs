use crate::domain::TemplateRepository;
use crate::domain::models::{Template, TemplateResponse};
use crate::error::{AppError, Result};
use std::sync::Arc;

pub struct TemplateUsecase {
    repo: Arc<dyn TemplateRepository>,
}

impl TemplateUsecase {
    pub fn new(repo: Arc<dyn TemplateRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_templates(&self) -> Result<Vec<TemplateResponse>> {
        let templates = self.repo.list_all().await?;
        Ok(templates.into_iter().map(TemplateResponse::from).collect())
    }

    pub async fn get_template(&self, id: &str) -> Result<Template> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Template {} not found", id)))
    }

    pub async fn create_template(&self, template: Template) -> Result<()> {
        self.repo.save(&template).await
    }

    pub async fn import_from_url(&self, url: &str) -> Result<Template> {
        let client = reqwest::Client::new();
        let template: Template = client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch template from URL: {}", e)))?
            .json()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to parse template JSON: {}", e)))?;

        self.repo.save(&template).await?;
        Ok(template)
    }

    pub async fn delete_template(&self, id: &str) -> Result<()> {
        self.repo.delete(id).await
    }

    pub async fn seed_default_templates(&self) -> Result<()> {
        let count = self.repo.count().await?;
        if count > 0 {
            return Ok(());
        }

        // Load templates from JSON files in template directory
        let template_dir = std::env::var("TEMPLATE_DIR").unwrap_or_else(|_| "template".to_string());

        let entries = match std::fs::read_dir(&template_dir) {
            Ok(entries) => entries,
            Err(e) => {
                tracing::warn!(
                    "Could not read template directory '{}': {}. Skipping template seeding.",
                    template_dir,
                    e
                );
                return Ok(());
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<Template>(&content) {
                        Ok(template) => {
                            tracing::info!("Loading template: {} from {:?}", template.name, path);
                            if let Err(e) = self.repo.save(&template).await {
                                tracing::warn!("Failed to save template {}: {}", template.id, e);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse template JSON {:?}: {}", path, e);
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to read template file {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }
}
