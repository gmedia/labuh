use reqwest::Client;
use std::sync::Arc;

use crate::domain::runtime::{ContainerConfig, RuntimePort};
use crate::error::{AppError, Result};

/// Version tag for Caddy container - increment to force re-creation
const CADDY_CONTAINER_VERSION: &str = "v3";
const LABUH_NETWORK: &str = "labuh-network";

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

    /// Ensure Caddy container is running with correct configuration
    pub async fn bootstrap(&self, runtime: &Arc<dyn RuntimePort>) -> Result<()> {
        let container_name = "labuh-caddy";

        // Ensure labuh-network exists
        runtime.ensure_network(LABUH_NETWORK).await?;

        // Check if running
        let containers = runtime.list_containers(true).await?;
        let existing = containers
            .iter()
            .find(|c| c.names.iter().any(|n| n.contains(container_name)));

        if let Some(c) = existing {
            // Check version label to determine if we need to recreate
            let needs_upgrade = !c
                .labels
                .get("labuh.caddy.version")
                .map(|v| v == CADDY_CONTAINER_VERSION)
                .unwrap_or(false);

            if needs_upgrade {
                tracing::info!(
                    "Upgrading Caddy container to {}...",
                    CADDY_CONTAINER_VERSION
                );
                let _ = runtime.stop_container(&c.id).await;
                let _ = runtime.remove_container(&c.id, true).await;
                // Fall through to create new container
            } else if c.state == "running" {
                return Ok(());
            } else {
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
            // Minimal Caddyfile - only admin API. Route config is done via JSON API.
            let default_caddyfile = r#"{
    admin 0.0.0.0:2019
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

        let mut labels = std::collections::HashMap::new();
        labels.insert("labuh.managed".to_string(), "true".to_string());
        labels.insert("labuh.service".to_string(), "caddy".to_string());
        labels.insert(
            "labuh.caddy.version".to_string(),
            CADDY_CONTAINER_VERSION.to_string(),
        );

        let config = ContainerConfig {
            name: container_name.to_string(),
            image: image.to_string(),
            ports: Some(port_bindings),
            volumes: Some(volumes),
            env: None,
            cmd: None,
            labels: Some(labels),
            cpu_limit: None,
            memory_limit: None,
            network_mode: Some(LABUH_NETWORK.to_string()),
            networks: None,
            extra_hosts: None,
            restart_policy: Some("always".to_string()),
        };

        let id = runtime.create_container(config).await?;

        tracing::info!("Starting Caddy container...");
        runtime.start_container(&id).await?;

        Ok(())
    }

    /// Initialize a new route
    pub async fn add_route(&self, domain: &str, upstream: &str, show_branding: bool) -> Result<()> {
        // Ensure srv0 structure exists first
        self.ensure_srv0().await?;

        // Remove existing route for this domain if it exists to prevent duplicates
        let _ = self.remove_route(domain).await;

        // Build handlers - always start with reverse_proxy
        let mut handlers: Vec<serde_json::Value> = vec![];

        if show_branding {
            // Badge HTML to inject before </body>
            let badge_html = r#"<div style="position:fixed;bottom:10px;right:10px;background:#1a1a2e;color:#fff;padding:8px 12px;border-radius:6px;font-size:12px;font-family:sans-serif;z-index:99999;opacity:0.9;box-shadow:0 2px 8px rgba(0,0,0,0.3);">⚡ Deployed with <a href="https://labuh.dev" target="_blank" style="color:#6366f1;text-decoration:none;font-weight:bold;">Labuh</a></div></body>"#;

            // Use subroute with reverse_proxy + replace_response
            handlers.push(serde_json::json!({
                "handler": "subroute",
                "routes": [{
                    "handle": [
                        {
                            "handler": "reverse_proxy",
                            "upstreams": [{ "dial": upstream }],
                            "handle_response": [{
                                "routes": [{
                                    "match": [{
                                        "header": {
                                            "Content-Type": ["*text/html*"]
                                        }
                                    }],
                                    "handle": [{
                                        "handler": "rewrite",
                                        "path_regexp": [{
                                            "find": "</body>",
                                            "replace": badge_html
                                        }]
                                    }]
                                }]
                            }]
                        }
                    ]
                }]
            }));
        } else {
            handlers.push(serde_json::json!({
                "handler": "reverse_proxy",
                "upstreams": [{ "dial": upstream }]
            }));
        }

        let route_config = serde_json::json!({
            "match": [{ "host": [domain] }],
            "handle": handlers
        });

        // Insert domain route at the BEGINNING (index 0) so it takes priority
        let response = self
            .request_with_fallback(
                reqwest::Method::POST,
                "/config/apps/http/servers/srv0/routes/0",
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

        tracing::info!(
            "Added route: {} -> {} (branding: {})",
            domain,
            upstream,
            show_branding
        );
        Ok(())
    }

    /// Ensure srv0 exists and is configured for HTTPS
    async fn ensure_srv0(&self) -> Result<()> {
        // Always set srv0 to listen on :443 for HTTPS
        let base_config = serde_json::json!({
            "listen": [":443"],
            "routes": []
        });

        let resp = self
            .request_with_fallback(reqwest::Method::GET, "/config/apps/http/servers/srv0", None)
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                // Check if already listening on 443
                let body: serde_json::Value = r.json().await.unwrap_or_default();
                let listen = body.get("listen").and_then(|l| l.as_array());
                let has_443 = listen
                    .map(|arr| {
                        arr.iter()
                            .any(|v| v.as_str().map(|s| s.contains("443")).unwrap_or(false))
                    })
                    .unwrap_or(false);

                if !has_443 {
                    tracing::info!("Upgrading srv0 to HTTPS (port 443)...");
                    // Get existing routes to preserve them
                    let routes = body.get("routes").cloned().unwrap_or(serde_json::json!([]));
                    let updated_config = serde_json::json!({
                        "listen": [":443"],
                        "routes": routes
                    });
                    self.request_with_fallback(
                        reqwest::Method::PUT,
                        "/config/apps/http/servers/srv0",
                        Some(updated_config),
                    )
                    .await?;
                }
                return Ok(());
            }
            _ => {
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
                if let Some(matches) = route.get("match")
                    && let Some(hosts) = matches.get(0).and_then(|m| m.get("host"))
                    && hosts
                        .as_array()
                        .map(|arr| arr.iter().any(|h| h.as_str() == Some(domain)))
                        .unwrap_or(false)
                {
                    index_to_remove = Some(index);
                    break;
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
