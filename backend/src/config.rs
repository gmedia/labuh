use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub caddy_admin_api: String,
    #[allow(dead_code)]
    pub caddy_config_path: String,
    #[allow(dead_code)]
    pub containerd_socket: String,
}

impl Config {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./labuh.db?mode=rwc".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")?,
            jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            caddy_admin_api: std::env::var("CADDY_ADMIN_API")
                .unwrap_or_else(|_| "http://localhost:2019".to_string()),
            caddy_config_path: std::env::var("CADDY_CONFIG_PATH")
                .unwrap_or_else(|_| "/etc/caddy/Caddyfile".to_string()),
            containerd_socket: std::env::var("CONTAINERD_SOCKET")
                .unwrap_or_else(|_| "/run/containerd/containerd.sock".to_string()),
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
