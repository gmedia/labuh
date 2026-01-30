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

    /// Helper to perform requests with localhost -> caddy fallback
    async fn request_with_fallback(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.admin_api_url, path);
        let mut builder = self.client.request(method.clone(), &url);
        if let Some(ref b) = body {
            builder = builder.json(b);
        }

        let resp = builder.send().await;

        match resp {
            Ok(r) => Ok(r),
            Err(e) if e.is_connect() && self.admin_api_url.contains("localhost") => {
                let fallback_url = self.admin_api_url.replace("localhost", "caddy");
                let fallback_full = format!("{}{}", fallback_url, path);
                let mut fallback_builder = self.client.request(method, &fallback_full);
                if let Some(ref b) = body {
                    fallback_builder = fallback_builder.json(b);
                }
                fallback_builder
                    .send()
                    .await
                    .map_err(|e| AppError::CaddyApi(e.to_string()))
            }
            Err(e) => Err(AppError::CaddyApi(e.to_string())),
        }
    }

    /// Ensure Caddy container is running
    pub async fn bootstrap(
        &self,
        container_service: &crate::services::ContainerService,
    ) -> Result<()> {
        let container_name = "labuh-caddy";

        // Check if running
        let containers = container_service.list_containers(true).await?;
        let existing = containers
            .iter()
            .find(|c| c.names.iter().any(|n| n.contains(container_name)));

        if let Some(c) = existing {
            // Check if port 2019 is bound (required for Admin API)
            let has_admin_port = c.ports.iter().any(|p| p.private_port == 2019);

            if c.state == "running" && has_admin_port {
                return Ok(());
            }

            if !has_admin_port {
                tracing::info!("Caddy is missing admin port 2019. Recreating...");
                let _ = container_service.stop_container(&c.id).await;
                let _ = container_service.remove_container(&c.id, true).await;
            } else if c.state != "running" {
                tracing::info!("Starting existing Caddy container...");
                container_service.start_container(&c.id).await?;
                return Ok(());
            }
        }

        tracing::info!("Creating Caddy container...");

        // Define Caddy container config
        let image = "caddy:2-alpine";

        // Ensure image exists
        container_service.pull_image(image).await?;

        // Ensure Caddyfile exists on host to avoid Docker creating it as a directory
        let caddyfile_path = std::env::current_dir()
            .unwrap_or_default()
            .join("Caddyfile");
        if !caddyfile_path.exists()
            || (caddyfile_path.is_file()
                && std::fs::metadata(&caddyfile_path)
                    .map(|m| m.len())
                    .unwrap_or(0)
                    == 0)
        {
            tracing::info!("Caddyfile not found. Creating default...");
            let default_caddyfile = r#"{
    admin 0.0.0.0:2019
}

:80 {
    handle /api/* {
        reverse_proxy labuh:3000
    }

    # Frontend fallback (Proxies to Labuh host-gateway for now)
    handle {
        reverse_proxy labuh:3000
    }
}
"#;
            std::fs::write(&caddyfile_path, default_caddyfile).map_err(|e| {
                AppError::Internal(format!("Failed to create default Caddyfile: {}", e))
            })?;
        } else if caddyfile_path.is_dir() {
            tracing::warn!("Caddyfile exists as a directory. Recreating as a file...");
            std::fs::remove_dir_all(&caddyfile_path).map_err(|e| {
                AppError::Internal(format!("Failed to remove Caddyfile directory: {}", e))
            })?;
            let default_caddyfile = r#"{
    admin 0.0.0.0:2019
}

:80 {
    handle /api/* {
        reverse_proxy labuh:3000
    }

    handle {
        reverse_proxy labuh:3000
    }
}
"#;
            std::fs::write(&caddyfile_path, default_caddyfile).map_err(|e| {
                AppError::Internal(format!("Failed to create default Caddyfile: {}", e))
            })?;
        }

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
        port_bindings.insert(
            "2019/tcp".to_string(),
            Some(vec![bollard::models::PortBinding {
                host_ip: Some("127.0.0.1".to_string()),
                host_port: Some("2019".to_string()),
            }]),
        );

        // Create container config
        let config = bollard::container::Config {
            image: Some(image.to_string()),
            cmd: Some(vec![
                "caddy".to_string(),
                "run".to_string(),
                "--config".to_string(),
                "/etc/caddy/Caddyfile".to_string(),
                "--adapter".to_string(),
                "caddyfile".to_string(),
            ]),
            exposed_ports: Some(HashMap::from([
                ("80/tcp".to_string(), HashMap::new()),
                ("443/tcp".to_string(), HashMap::new()),
                ("2019/tcp".to_string(), HashMap::new()),
            ])),
            host_config: Some(bollard::service::HostConfig {
                network_mode: Some("bridge".to_string()),
                port_bindings: Some(port_bindings),
                extra_hosts: Some(vec![
                    "labuh:host-gateway".to_string(),
                    "frontend:host-gateway".to_string(),
                ]),
                binds: Some(vec![
                    format!(
                        "{}/Caddyfile:/etc/caddy/Caddyfile",
                        std::env::current_dir().unwrap().to_string_lossy()
                    ),
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

        container_service
            .docker
            .create_container(Some(options), config)
            .await
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
        // Convert Caddyfile to Caddy JSON config
        let response = self
            .client
            .post(format!("{}/adapt", self.admin_api_url))
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
            .request_with_fallback(reqwest::Method::POST, "/load", Some(json_config))
            .await?;

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

    /// Initialize a new route
    pub async fn add_route(&self, domain: &str, upstream: &str) -> Result<()> {
        // Ensure srv0 structure exists first
        self.ensure_srv0().await?;

        // Remove existing route for this domain if it exists to prevent duplicates
        let _ = self.remove_route(domain).await;

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

        let response = self
            .request_with_fallback(
                reqwest::Method::POST,
                "/config/apps/http/servers/srv0/routes",
                Some(route_config.clone()),
            )
            .await?;

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

    /// Ensure srv0 exists in Caddy JSON config
    async fn ensure_srv0(&self) -> Result<()> {
        let resp = self
            .request_with_fallback(reqwest::Method::GET, "/config/apps/http/servers/srv0", None)
            .await;

        match resp {
            Ok(r) if r.status().is_success() => return Ok(()),
            _ => {
                // Initialize basic srv0 if it doesn't exist
                let base_config = serde_json::json!({
                    "listen": [":80"],
                    "routes": []
                });

                let init_resp = self
                    .request_with_fallback(
                        reqwest::Method::PUT,
                        "/config/apps/http/servers/srv0",
                        Some(base_config.clone()),
                    )
                    .await?;

                if !init_resp.status().is_success() {
                    // If PUT fails, try PUT to apps/http first
                    let http_config = serde_json::json!({
                        "servers": {
                            "srv0": base_config
                        }
                    });
                    self.request_with_fallback(
                        reqwest::Method::PUT,
                        "/config/apps/http",
                        Some(http_config),
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }

    /// Remove all routes matching a domain
    pub async fn remove_route(&self, domain: &str) -> Result<()> {
        let mut found_any = false;

        loop {
            let response = self
                .request_with_fallback(
                    reqwest::Method::GET,
                    "/config/apps/http/servers/srv0/routes",
                    None,
                )
                .await?;

            if !response.status().is_success() {
                break;
            }

            let routes: Vec<serde_json::Value> = response
                .json()
                .await
                .map_err(|e| AppError::CaddyApi(e.to_string()))?;

            let mut index_to_remove = None;
            for (index, route) in routes.iter().enumerate() {
                if let Some(matches) = route.get("match") {
                    if let Some(hosts) = matches.get(0).and_then(|m| m.get("host")) {
                        if hosts
                            .as_array()
                            .map(|arr| arr.iter().any(|h| h.as_str() == Some(domain)))
                            .unwrap_or(false)
                        {
                            index_to_remove = Some(index);
                            break;
                        }
                    }
                }
            }

            if let Some(index) = index_to_remove {
                self.request_with_fallback(
                    reqwest::Method::DELETE,
                    &format!("/config/apps/http/servers/srv0/routes/{}", index),
                    None,
                )
                .await?;
                found_any = true;
            } else {
                break;
            }
        }

        if found_any {
            Ok(())
        } else {
            Err(AppError::NotFound(format!(
                "Route for domain {} not found",
                domain
            )))
        }
    }

    /// Add a route with Basic Auth protection
    #[allow(dead_code)]
    pub async fn add_route_with_basic_auth(
        &self,
        domain: &str,
        upstream: &str,
        username: &str,
        password_hash: &str,
    ) -> Result<()> {
        self.ensure_srv0().await?;

        let route_config = serde_json::json!({
            "match": [{
                "host": [domain]
            }],
            "handle": [
                {
                    "handler": "authentication",
                    "providers": {
                        "http_basic": {
                            "accounts": [
                                {
                                    "username": username,
                                    "password": password_hash
                                }
                            ],
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

        let response = self
            .request_with_fallback(
                reqwest::Method::POST,
                "/config/apps/http/servers/srv0/routes",
                Some(route_config),
            )
            .await?;

        if !response.status().is_success() {
            let error_text = response.status().to_string();
            return Err(AppError::CaddyApi(format!(
                "Failed to add route with basic auth: {}",
                error_text
            )));
        }

        tracing::info!("Added route with basic auth: {} -> {}", domain, upstream);
        Ok(())
    }
}
