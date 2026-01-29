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
#[allow(dead_code)]
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

    /// Ensure Caddy container is running
    pub async fn bootstrap(&self, container_service: &crate::services::ContainerService) -> Result<()> {
        let container_name = "labuh-caddy";

        // Check if running
        let containers = container_service.list_containers(true).await?;
        let existing = containers.iter().find(|c| c.names.iter().any(|n| n.contains(container_name)));

        if let Some(c) = existing {
            if c.state != "running" {
                tracing::info!("Starting existing Caddy container...");
                container_service.start_container(&c.id).await?;
            }
            return Ok(());
        }

        tracing::info!("Creating Caddy container...");

        // Define Caddy container config
        let image = "caddy:2-alpine";

        // Ensure image exists
        container_service.pull_image(image).await?;

        // Setup port bindings
        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            "80/tcp".to_string(),
            Some(vec![bollard::models::PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some("80".to_string()),
            }]),
        );
        port_bindings.insert(
            "443/tcp".to_string(),
            Some(vec![bollard::models::PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some("443".to_string()),
            }]),
        );

        // Create container config
        let config = bollard::container::Config {
            image: Some(image.to_string()),
            cmd: Some(vec![
                "caddy".to_string(), "run".to_string(),
                "--config".to_string(), "/etc/caddy/Caddyfile".to_string(),
                "--adapter".to_string(), "caddyfile".to_string(),
            ]),
            exposed_ports: Some(HashMap::from([
                ("80/tcp".to_string(), HashMap::new()),
                ("443/tcp".to_string(), HashMap::new()),
            ])),
            host_config: Some(bollard::service::HostConfig {
                network_mode: Some("bridge".to_string()),
                port_bindings: Some(port_bindings),
                extra_hosts: Some(vec![
                    "labuh:host-gateway".to_string(),
                    "frontend:host-gateway".to_string(),
                ]),
                binds: Some(vec![
                    format!("{}/Caddyfile:/etc/caddy/Caddyfile", std::env::current_dir().unwrap().to_string_lossy()),
                    "caddy_data:/data".to_string(),
                    "caddy_config:/config".to_string(),
                ]),
                restart_policy: Some(bollard::service::RestartPolicy {
                    name: Some(bollard::service::RestartPolicyNameEnum::UNLESS_STOPPED),
                    maximum_retry_count: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let options = bollard::container::CreateContainerOptions {
            name: container_name,
            ..Default::default()
        };

        container_service.docker.create_container(Some(options), config).await
            .map_err(|e| crate::error::AppError::ContainerRuntime(e.to_string()))?;

        tracing::info!("Starting Caddy container...");
        container_service.start_container(container_name).await?;

        Ok(())
    }

    /// Generate Caddyfile content from routes
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    /// Add a route with Basic Auth protection
    pub async fn add_route_with_basic_auth(
        &self,
        domain: &str,
        upstream: &str,
        username: &str,
        password_hash: &str,
    ) -> Result<()> {
        // Caddy expects bcrypt-hashed passwords for basic auth
        let route_config = serde_json::json!({
            "match": [{
                "host": [domain]
            }],
            "handle": [
                {
                    "handler": "authentication",
                    "providers": {
                        "http_basic": {
                            "accounts": [{
                                "username": username,
                                "password": password_hash
                            }],
                            "hash": {
                                "algorithm": "bcrypt"
                            }
                        }
                    }
                },
                {
                    "handler": "reverse_proxy",
                    "upstreams": [{
                        "dial": upstream
                    }]
                }
            ]
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
                "Failed to add authenticated route: {}",
                error_text
            )));
        }

        tracing::info!("Added authenticated route: {} -> {} (user: {})", domain, upstream, username);
        Ok(())
    }
}
