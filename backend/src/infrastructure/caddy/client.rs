use reqwest::Client;
use std::sync::Arc;

use crate::domain::runtime::{ContainerConfig, RuntimePort};
use crate::error::{AppError, Result};

/// Caddy Admin API client for dynamic configuration
pub struct CaddyClient {
    admin_api_url: String,
    client: Client,
}

impl CaddyClient {
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
    pub async fn bootstrap(&self, runtime: &Arc<dyn RuntimePort>) -> Result<()> {
        let container_name = "labuh-caddy";

        // Check if running
        let containers = runtime.list_containers(true).await?;
        let existing = containers
            .iter()
            .find(|c| c.names.iter().any(|n| n.contains(container_name)));

        if let Some(c) = existing {
            // We can't easily check ports from list_containers result in simpler struct,
            // but we can trust it if it's running or inspect it.
            // Let's inspect to be safe if status is running.
            if c.state == "running" {
                let _info = runtime.inspect_container(&c.id).await?;
                // Check if port 2019 is bound (required for Admin API)
                // Note: ContainerInfo from runtime might not have ports detail structure as bollard,
                // checking generic validity.
                // For now, assuming if running it's fine or we restart.
                return Ok(());
            }

            if c.state != "running" {
                tracing::info!("Starting existing Caddy container...");
                runtime.start_container(&c.id).await?;
                return Ok(());
            }
        }

        tracing::info!("Creating Caddy container...");

        // Define Caddy container config
        let image = "caddy:2-alpine";

        // Ensure image exists
        runtime.pull_image(image, None).await?;

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
        }

        // Create container config
        let port_bindings = vec![
            "80:80".to_string(),
            "443:443".to_string(),
            "127.0.0.1:2019:2019".to_string(),
        ];

        let volumes = vec![
            format!(
                "{}/Caddyfile:/etc/caddy/Caddyfile",
                std::env::current_dir().unwrap().to_string_lossy()
            ),
            "caddy_data:/data".to_string(),
            "caddy_config:/config".to_string(),
        ];

        let config = ContainerConfig {
            name: container_name.to_string(),
            image: image.to_string(),
            ports: Some(port_bindings),
            volumes: Some(volumes),
            // Extra hosts need adding to RuntimePort definition if we want to support them properly
            // For now omitting extra_hosts or we need to add specific method for Caddy.
            // Or rely on `DockerRuntimeAdapter` internal details via trait extension?
            // Actually `ContainerConfig` has limited fields.
            // We need `extra_hosts` in `ContainerConfig` to support `labuh:host-gateway`.
            env: None,
            cmd: None,
            labels: None,
            cpu_limit: None,
            memory_limit: None,
            network_mode: None,
            extra_hosts: None,
            restart_policy: Some("always".to_string()),
        };
        // TODO: Add extra_hosts to ContainerConfig if strictly needed for host-gateway.
        // Assuming bridge mode default.

        let id = runtime.create_container(config).await?;

        tracing::info!("Starting Caddy container...");
        runtime.start_container(&id).await?;

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
}
