use crate::error::{AppError, Result};
use std::path::Path;
use tokio::process::Command;
use tracing::info;

pub struct GitService;

impl GitService {
    pub fn new() -> Self {
        Self
    }

    pub async fn clone_or_pull(&self, url: &str, branch: &str, target_dir: &str) -> Result<String> {
        let path = Path::new(target_dir);
        let branch = if branch.is_empty() { "main" } else { branch };

        if path.exists() {
            info!("Pulling existing repository at {}", target_dir);
            let output = Command::new("git")
                .arg("-C")
                .arg(target_dir)
                .arg("pull")
                .arg("origin")
                .arg(branch)
                .output()
                .await
                .map_err(|e| AppError::Internal(format!("Failed to execute git pull: {}", e)))?;

            if !output.status.success() {
                return Err(AppError::Internal(format!(
                    "Git pull failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
        } else {
            info!("Cloning repository {} into {}", url, target_dir);
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    AppError::Internal(format!("Failed to create git data directory: {}", e))
                })?;
            }

            let output = Command::new("git")
                .arg("clone")
                .arg("-b")
                .arg(branch)
                .arg(url)
                .arg(target_dir)
                .output()
                .await
                .map_err(|e| AppError::Internal(format!("Failed to execute git clone: {}", e)))?;

            if !output.status.success() {
                return Err(AppError::Internal(format!(
                    "Git clone failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
        }

        // Get last commit hash
        let output = Command::new("git")
            .arg("-C")
            .arg(target_dir)
            .arg("rev-parse")
            .arg("HEAD")
            .output()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get commit hash: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Internal("Failed to get commit hash".to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
