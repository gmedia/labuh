use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{AppError, Result};

/// Caddy Admin API client for dynamic configuration
pub struct CaddyService {
    admin_api_url: String,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaddyRoute {
    pub domain: String,
    pub upstream: String,
}

impl CaddyService {
    pub fn new(admin_api_url: String) -> Self {
        Self {
            admin_api_url,
            client: Client::new(),
        }
    }

    /// Generate Caddyfile content from routes
    pub fn generate_caddyfile(
        &self,
        routes: &[CaddyRoute],
        api_upstream: &str,
        frontend_upstream: &str,
    ) -> String {
        let mut content = String::from(
            r#"# Labuh Platform - Auto-generated Caddyfile
{
    admin 0.0.0.0:2019
}

"#,
        );

        // Main site with API and frontend
        content.push_str(&format!(
            r#":80 {{
    handle /api/* {{
        reverse_proxy {}
    }}

    handle {{
        reverse_proxy {}
    }}
}}

"#,
            api_upstream, frontend_upstream
        ));

        // Dynamic routes for deployed apps
        for route in routes {
            content.push_str(&format!(
                r#"{} {{
    reverse_proxy {}
}}

"#,
                route.domain, route.upstream
            ));
        }

        content
    }

    /// Reload Caddy configuration via Admin API
    pub async fn reload_config(&self, caddyfile_content: &str) -> Result<()> {
        let url = format!("{}/load", self.admin_api_url);

        // Convert Caddyfile to Caddy JSON config
        let response = self
            .client
            .post(&format!("{}/adapt", self.admin_api_url))
            .header("Content-Type", "text/caddyfile")
            .body(caddyfile_content.to_string())
            .send()
            .await
            .map_err(|e| AppError::CaddyApi(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::CaddyApi(format!(
                "Failed to adapt Caddyfile: {}",
                error_text
            )));
        }

        let json_config: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::CaddyApi(e.to_string()))?;

        // Load the JSON config
        let load_response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json_config)
            .send()
            .await
            .map_err(|e| AppError::CaddyApi(e.to_string()))?;

        if !load_response.status().is_success() {
            let error_text = load_response.text().await.unwrap_or_default();
            return Err(AppError::CaddyApi(format!(
                "Failed to load config: {}",
                error_text
            )));
        }

        tracing::info!("Caddy configuration reloaded successfully");
        Ok(())
    }

    /// Add a new route dynamically
    pub async fn add_route(&self, domain: &str, upstream: &str) -> Result<()> {
        let route_config = serde_json::json!({
            "match": [{
                "host": [domain]
            }],
            "handle": [{
                "handler": "reverse_proxy",
                "upstreams": [{
                    "dial": upstream
                }]
            }]
        });

        let url = format!(
            "{}/config/apps/http/servers/srv0/routes",
            self.admin_api_url
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&route_config)
            .send()
            .await
            .map_err(|e| AppError::CaddyApi(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::CaddyApi(format!(
                "Failed to add route: {}",
                error_text
            )));
        }

        tracing::info!("Added route: {} -> {}", domain, upstream);
        Ok(())
    }

    /// Remove a route by domain
    pub async fn remove_route(&self, domain: &str) -> Result<()> {
        // Get current config first
        let url = format!(
            "{}/config/apps/http/servers/srv0/routes",
            self.admin_api_url
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::CaddyApi(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::CaddyApi(format!(
                "Failed to get routes: {}",
                error_text
            )));
        }

        let routes: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| AppError::CaddyApi(e.to_string()))?;

        // Find and remove the route with matching domain
        for (index, route) in routes.iter().enumerate() {
            if let Some(matches) = route.get("match") {
                if let Some(hosts) = matches.get(0).and_then(|m| m.get("host")) {
                    if hosts
                        .as_array()
                        .map(|arr| arr.iter().any(|h| h.as_str() == Some(domain)))
                        .unwrap_or(false)
                    {
                        let delete_url = format!("{}/{}", url, index);
                        self.client
                            .delete(&delete_url)
                            .send()
                            .await
                            .map_err(|e| AppError::CaddyApi(e.to_string()))?;

                        tracing::info!("Removed route for domain: {}", domain);
                        return Ok(());
                    }
                }
            }
        }

        Err(AppError::NotFound(format!(
            "Route for domain {} not found",
            domain
        )))
    }
}
