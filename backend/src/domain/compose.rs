//! Docker Compose file parser
//!
//! Parses docker-compose.yml and converts to container creation requests.

use serde::Deserialize;
use std::collections::HashMap;

use crate::domain::runtime::ContainerConfig;
use crate::error::{AppError, Result};

/// Parsed Docker Compose file structure
#[derive(Debug, Deserialize)]
pub struct ComposeFile {
    pub services: HashMap<String, ComposeService>,
    #[serde(default)]
    pub networks: HashMap<String, ComposeNetwork>,
}

#[derive(Debug, Deserialize)]
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
    pub _container_name: Option<String>,
    pub _restart: Option<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    pub deploy: Option<ComposeDeploy>,
}

#[derive(Debug, Deserialize)]
pub struct ComposeDeploy {
    pub replicas: Option<u32>,
    pub resources: Option<ComposeResources>,
    #[serde(default)]
    pub placement: ComposePlacement,
}

#[derive(Debug, Deserialize, Default)]
pub struct ComposePlacement {
    #[serde(default)]
    pub constraints: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ComposeResources {
    pub limits: Option<ComposeLimits>,
}

#[derive(Debug, Deserialize)]
pub struct ComposeLimits {
    pub cpus: Option<String>,
    pub memory: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
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
    Map(HashMap<String, serde_yaml::Value>),
}

#[derive(Debug, Deserialize, Default)]
pub struct ComposeNetwork {}

/// Result of parsing a compose file
#[derive(Debug)]
pub struct ParsedCompose {
    pub services: Vec<ParsedService>,
    pub networks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedService {
    pub labels: std::collections::HashMap<String, String>,
    pub name: String,
    pub image: String,
    pub env: Vec<String>,
    pub ports: HashMap<String, String>,
    pub volumes: HashMap<String, String>,
    pub depends_on: Vec<String>,
    pub networks: Vec<String>,
    pub build: Option<ParsedBuild>,
    pub cpu_limit: Option<f64>,
    pub memory_limit: Option<i64>,
    pub deploy: Option<ParsedDeploy>,
}

#[derive(Debug, Clone)]
pub struct ParsedDeploy {
    pub replicas: Option<u32>,
    pub placement: ParsedPlacement,
}

#[derive(Debug, Clone)]
pub struct ParsedPlacement {
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedBuild {
    pub context: String,
    pub dockerfile: String,
}

/// Blocked host paths that should never be mounted
const BLOCKED_HOST_PATHS: &[&str] = &[
    "/", "/bin", "/boot", "/dev", "/etc", "/home", "/lib", "/lib64", "/opt", "/proc", "/root",
    "/run", "/sbin", "/sys", "/tmp", "/usr", "/var",
];

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
                "Mounting root filesystem (/) is not allowed".to_string(),
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
        // Parse build context
        let build = match &service.build {
            Some(ComposeBuild::Simple(context)) => Some(ParsedBuild {
                context: context.clone(),
                dockerfile: "Dockerfile".to_string(),
            }),
            Some(ComposeBuild::Extended {
                context,
                dockerfile,
            }) => Some(ParsedBuild {
                context: context.clone(),
                dockerfile: dockerfile
                    .clone()
                    .unwrap_or_else(|| "Dockerfile".to_string()),
            }),
            None => None,
        };

        // Get image (required if no build, or can be specified with build as tag)
        let image = match (&service.image, &build) {
            (Some(img), _) => img.clone(),
            (None, Some(_)) => {
                // If no image name is provided but build is, we use the service name as the image name
                format!("labuh-local/{}", name)
            }
            (None, None) => {
                return Err(AppError::Validation(format!(
                    "Service '{}' must have an image or build context",
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
                .filter_map(|(k, v)| {
                    let val_str = match v {
                        serde_yaml::Value::Null => return None,
                        serde_yaml::Value::Bool(b) => b.to_string(),
                        serde_yaml::Value::Number(n) => n.to_string(),
                        serde_yaml::Value::String(s) => s,
                        _ => return None, // Skip complex types
                    };
                    Some(format!("{}={}", k, val_str))
                })
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

        // Parse limits
        let mut cpu_limit = None;
        let mut memory_limit = None;
        let mut replicas = None;
        let mut constraints = Vec::new();

        if let Some(deploy) = &service.deploy {
            replicas = deploy.replicas;
            constraints = deploy.placement.constraints.clone();

            if let Some(limits) = deploy.resources.as_ref().and_then(|r| r.limits.as_ref()) {
                if let Some(cpus) = &limits.cpus {
                    cpu_limit = cpus.parse::<f64>().ok();
                }
                if let Some(memory) = &limits.memory {
                    memory_limit = parse_memory(memory);
                }
            }
        }

        services.push(ParsedService {
            name,
            image,
            env,
            ports,
            volumes,
            depends_on: service.depends_on,
            networks: service.networks,
            labels: service.labels,
            build,
            cpu_limit,
            memory_limit,
            deploy: Some(ParsedDeploy {
                replicas,
                placement: ParsedPlacement { constraints },
            }),
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

fn parse_memory(memory: &str) -> Option<i64> {
    let memory = memory.to_uppercase();
    // Support formats like "256M", "1024", "1G"
    let val_str: String = memory.chars().filter(|c| c.is_numeric()).collect();
    let val = val_str.parse::<i64>().ok()?;

    if memory.contains('G') {
        Some(val * 1024 * 1024 * 1024)
    } else if memory.contains('M') {
        Some(val * 1024 * 1024)
    } else if memory.contains('K') {
        Some(val * 1024)
    } else {
        Some(val)
    }
}

/// Convert parsed service to container creation request
pub fn service_to_container_request(
    service: &ParsedService,
    stack_id: &str,
    stack_name: &str,
) -> ContainerConfig {
    let mut labels = service.labels.clone();
    labels.insert("labuh.stack.id".to_string(), stack_id.to_string());
    labels.insert("labuh.stack.name".to_string(), stack_name.to_string());
    labels.insert("labuh.service.name".to_string(), service.name.clone());

    // Convert port HashMap<container, host> to Vec<"host:container">
    let ports: Option<Vec<String>> = if service.ports.is_empty() {
        None
    } else {
        Some(
            service
                .ports
                .iter()
                .map(|(c, h)| format!("{}:{}", h, c))
                .collect(),
        )
    };

    // Convert volume HashMap<host, container> to Vec<"host:container">
    let volumes: Option<Vec<String>> = if service.volumes.is_empty() {
        None
    } else {
        Some(
            service
                .volumes
                .iter()
                .map(|(h, c)| format!("{}:{}", h, c))
                .collect(),
        )
    };

    ContainerConfig {
        name: format!("{}-{}", stack_name, service.name),
        image: service.image.clone(),
        env: if service.env.is_empty() {
            None
        } else {
            Some(service.env.clone())
        },
        ports,
        volumes,
        labels: Some(labels),
        cpu_limit: service.cpu_limit,
        memory_limit: service.memory_limit,
        cmd: None,
        network_mode: Some("labuh-network".to_string()),
        extra_hosts: None,
        restart_policy: Some("unless-stopped".to_string()),
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
