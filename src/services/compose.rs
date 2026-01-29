//! Docker Compose file parser
//!
//! Parses docker-compose.yml and converts to container creation requests.

use serde::Deserialize;
use std::collections::HashMap;

use crate::error::{AppError, Result};
use crate::services::container::CreateContainerRequest;

/// Parsed Docker Compose file structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ComposeFile {
    pub version: Option<String>,
    pub services: HashMap<String, ComposeService>,
    #[serde(default)]
    pub networks: HashMap<String, ComposeNetwork>,
    #[serde(default)]
    pub volumes: HashMap<String, ComposeVolume>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ComposeService {
    pub image: Option<String>,
    pub build: Option<ComposeBuild>,
    #[serde(default)]
    pub environment: ComposeEnvironment,
    #[serde(default)]
    pub ports: Vec<String>,
    #[serde(default)]
    pub volumes: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub networks: Vec<String>,
    pub container_name: Option<String>,
    pub restart: Option<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    pub command: Option<ComposeCommand>,
    pub entrypoint: Option<ComposeCommand>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum ComposeBuild {
    Simple(String),
    Extended {
        context: String,
        dockerfile: Option<String>,
    },
}

#[derive(Debug, Deserialize, Default)]
#[serde(untagged)]
pub enum ComposeEnvironment {
    #[default]
    Empty,
    List(Vec<String>),
    Map(HashMap<String, Option<String>>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum ComposeCommand {
    Simple(String),
    List(Vec<String>),
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct ComposeNetwork {
    pub driver: Option<String>,
    pub external: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct ComposeVolume {
    pub driver: Option<String>,
    pub external: Option<bool>,
}

/// Result of parsing a compose file
#[derive(Debug)]
#[allow(dead_code)]
pub struct ParsedCompose {
    pub services: Vec<ParsedService>,
    pub networks: Vec<String>,
}

#[derive(Debug)]
pub struct ParsedService {
    pub name: String,
    pub image: String,
    pub env: Vec<String>,
    pub ports: HashMap<String, String>,
    pub volumes: HashMap<String, String>,
    pub depends_on: Vec<String>,
}

/// Blocked host paths that should never be mounted
const BLOCKED_HOST_PATHS: &[&str] = &[
    "/",
    "/bin",
    "/boot",
    "/dev",
    "/etc",
    "/home",
    "/lib",
    "/lib64",
    "/opt",
    "/proc",
    "/root",
    "/run",
    "/sbin",
    "/sys",
    "/tmp",
    "/usr",
    "/var",
];

/// Security validation result
#[derive(Debug)]
#[allow(dead_code)]
pub struct VolumeValidationError {
    pub volume: String,
    pub reason: String,
}

/// Validate volume mounts for security issues
pub fn validate_volume_security(volumes: &[String]) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    for vol in volumes {
        let parts: Vec<&str> = vol.split(':').collect();
        if parts.len() < 2 {
            continue;
        }

        let host_path = parts[0];

        // Check for path traversal
        if host_path.contains("..") {
            return Err(AppError::Validation(format!(
                "Volume '{}' contains path traversal (..) which is not allowed",
                vol
            )));
        }

        // Check for root mount
        if host_path == "/" {
            return Err(AppError::Validation(
                "Mounting root filesystem (/) is not allowed".to_string()
            ));
        }

        // Skip named volumes (no / prefix)
        if !host_path.starts_with('/') && !host_path.starts_with('.') {
            continue;
        }

        // Normalize relative paths
        let normalized = if host_path.starts_with("./") {
            // Relative paths are allowed but warn
            warnings.push(format!("Volume '{}' uses relative path", vol));
            continue;
        } else {
            host_path.to_string()
        };

        // Check against blocked paths
        for blocked in BLOCKED_HOST_PATHS {
            if normalized == *blocked || normalized.starts_with(&format!("{}/", blocked)) {
                return Err(AppError::Validation(format!(
                    "Volume '{}' mounts sensitive path '{}' which is not allowed",
                    vol, blocked
                )));
            }
        }

        // Warn on absolute paths (they bypass relative sandboxing)
        if host_path.starts_with('/') {
            warnings.push(format!(
                "Volume '{}' uses absolute host path - ensure this is intentional",
                vol
            ));
        }
    }

    Ok(warnings)
}

/// Parse docker-compose.yml content
pub fn parse_compose(yaml_content: &str) -> Result<ParsedCompose> {
    let compose: ComposeFile = serde_yaml::from_str(yaml_content)
        .map_err(|e| AppError::Validation(format!("Invalid compose file: {}", e)))?;

    let mut services = Vec::new();

    for (name, service) in compose.services {
        // Get image (required for now, we don't support build)
        let image = match (&service.image, &service.build) {
            (Some(img), _) => img.clone(),
            (None, Some(_)) => {
                return Err(AppError::Validation(format!(
                    "Service '{}' uses build context which is not supported. Please use a pre-built image.",
                    name
                )));
            }
            (None, None) => {
                return Err(AppError::Validation(format!(
                    "Service '{}' must have an image",
                    name
                )));
            }
        };

        // Parse environment variables
        let env = match service.environment {
            ComposeEnvironment::Empty => vec![],
            ComposeEnvironment::List(list) => list,
            ComposeEnvironment::Map(map) => map
                .into_iter()
                .filter_map(|(k, v)| v.map(|val| format!("{}={}", k, val)))
                .collect(),
        };

        // Parse ports (format: "8080:80" or "8080:80/tcp")
        let mut ports = HashMap::new();
        for port_str in service.ports {
            let parts: Vec<&str> = port_str.split(':').collect();
            if parts.len() == 2 {
                let host_port = parts[0].to_string();
                let container_port = parts[1].split('/').next().unwrap_or(parts[1]).to_string();
                ports.insert(container_port, host_port);
            }
        }

        // Validate volume security before parsing
        let volume_warnings = validate_volume_security(&service.volumes)?;
        for warning in volume_warnings {
            tracing::warn!("Compose validation: {}", warning);
        }

        // Parse volumes (format: "./data:/app/data" or "volume_name:/app/data")
        let mut volumes = HashMap::new();
        for vol_str in service.volumes {
            let parts: Vec<&str> = vol_str.split(':').collect();
            if parts.len() >= 2 {
                volumes.insert(parts[0].to_string(), parts[1].to_string());
            }
        }

        services.push(ParsedService {
            name,
            image,
            env,
            ports,
            volumes,
            depends_on: service.depends_on,
        });
    }

    // Sort services by dependencies (simple topological sort)
    services.sort_by(|a, b| {
        if a.depends_on.contains(&b.name) {
            std::cmp::Ordering::Greater
        } else if b.depends_on.contains(&a.name) {
            std::cmp::Ordering::Less
        } else {
            a.name.cmp(&b.name)
        }
    });

    let networks: Vec<String> = compose.networks.keys().cloned().collect();

    Ok(ParsedCompose { services, networks })
}

/// Convert parsed service to container creation request
pub fn service_to_container_request(
    service: &ParsedService,
    stack_id: &str,
    stack_name: &str,
) -> CreateContainerRequest {
    let mut labels = HashMap::new();
    labels.insert("labuh.stack.id".to_string(), stack_id.to_string());
    labels.insert("labuh.stack.name".to_string(), stack_name.to_string());
    labels.insert("labuh.service.name".to_string(), service.name.clone());

    CreateContainerRequest {
        name: format!("{}-{}", stack_name, service.name),
        image: service.image.clone(),
        env: if service.env.is_empty() { None } else { Some(service.env.clone()) },
        ports: if service.ports.is_empty() { None } else { Some(service.ports.clone()) },
        volumes: if service.volumes.is_empty() { None } else { Some(service.volumes.clone()) },
        network: Some("labuh-network".to_string()),
        labels: Some(labels),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_root_mount() {
        let volumes = vec!["/:/container".to_string()];
        let result = validate_volume_security(&volumes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("root filesystem"));
    }

    #[test]
    fn test_block_etc_mount() {
        let volumes = vec!["/etc:/etc".to_string()];
        let result = validate_volume_security(&volumes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("/etc"));
    }

    #[test]
    fn test_block_path_traversal() {
        let volumes = vec!["../../../etc:/etc".to_string()];
        let result = validate_volume_security(&volumes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path traversal"));
    }

    #[test]
    fn test_allow_named_volumes() {
        let volumes = vec!["postgres_data:/var/lib/postgresql/data".to_string()];
        let result = validate_volume_security(&volumes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_allow_relative_paths() {
        let volumes = vec!["./data:/app/data".to_string()];
        let result = validate_volume_security(&volumes);
        assert!(result.is_ok());
        // Should have a warning
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_block_var_mount() {
        let volumes = vec!["/var/log:/logs".to_string()];
        let result = validate_volume_security(&volumes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_compose_with_dangerous_volume() {
        let yaml = r#"
version: "3"
services:
  evil:
    image: alpine
    volumes:
      - /:/host
"#;
        let result = parse_compose(yaml);
        assert!(result.is_err());
    }
}
