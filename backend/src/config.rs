use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_database_url")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable session-based authentication (login/register)
    #[serde(default = "default_enable_session")]
    pub enable_session: bool,

    /// Enable header-based authentication (reverse proxy)
    #[serde(default = "default_enable_header")]
    pub enable_header: bool,

    /// Header name to use for header-based auth
    #[serde(default = "default_header_name")]
    pub header_name: String,

    /// Enable HTTP Basic Authentication
    #[serde(default = "default_enable_basic")]
    pub enable_basic: bool,

    /// Disable all authentication (USE WITH CAUTION!)
    #[serde(default = "default_disable_all")]
    pub disable_all: bool,
}

// Default values
fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_database_url() -> String {
    "sqlite:llmproxy.db".to_string()
}

fn default_enable_session() -> bool {
    true
}

fn default_enable_header() -> bool {
    true
}

fn default_header_name() -> String {
    "Remote-User".to_string()
}

fn default_enable_basic() -> bool {
    false
}

fn default_disable_all() -> bool {
    false
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: default_database_url(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enable_session: default_enable_session(),
            enable_header: default_enable_header(),
            header_name: default_header_name(),
            enable_basic: default_enable_basic(),
            disable_all: default_disable_all(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            auth: AuthConfig::default(),
        }
    }
}

impl Config {
    /// Load config from file, with fallback to defaults and environment variables
    pub fn load() -> Result<Self> {
        let config_path =
            std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());

        let config = if Path::new(&config_path).exists() {
            tracing::info!("Loading config from {}", config_path);
            let content = fs::read_to_string(&config_path)
                .context(format!("Failed to read config file: {}", config_path))?;

            toml::from_str::<Config>(&content).context("Failed to parse config file")?
        } else {
            tracing::info!("Config file not found at {}, using defaults", config_path);
            Config::default()
        };

        // Override with environment variables if present
        let mut final_config = config;

        if let Ok(host) = std::env::var("HOST") {
            final_config.server.host = host;
        }

        if let Ok(port) = std::env::var("PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                final_config.server.port = port_num;
            }
        }

        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            final_config.database.url = database_url;
        }

        if let Ok(val) = std::env::var("AUTH_ENABLE_SESSION") {
            final_config.auth.enable_session = val.to_lowercase() == "true";
        }

        if let Ok(val) = std::env::var("AUTH_ENABLE_HEADER") {
            final_config.auth.enable_header = val.to_lowercase() == "true";
        }

        if let Ok(header_name) = std::env::var("AUTH_HEADER_NAME") {
            final_config.auth.header_name = header_name;
        }

        if let Ok(val) = std::env::var("AUTH_ENABLE_BASIC") {
            final_config.auth.enable_basic = val.to_lowercase() == "true";
        }

        if let Ok(val) = std::env::var("AUTH_DISABLE_ALL") {
            final_config.auth.disable_all = val.to_lowercase() == "true";
        }

        tracing::info!("Loaded configuration:");
        tracing::info!(
            "  Server: {}:{}",
            final_config.server.host,
            final_config.server.port
        );
        tracing::info!("  Database: {}", final_config.database.url);
        tracing::info!("  Auth - Session: {}", final_config.auth.enable_session);
        tracing::info!(
            "  Auth - Header: {} ({})",
            final_config.auth.enable_header,
            final_config.auth.header_name
        );
        tracing::info!("  Auth - Basic: {}", final_config.auth.enable_basic);
        tracing::info!("  Auth - Disable All: {}", final_config.auth.disable_all);

        if final_config.auth.disable_all {
            tracing::warn!("⚠️  WARNING: Authentication is COMPLETELY DISABLED! This is insecure!");
        }

        if !final_config.auth.enable_session
            && !final_config.auth.enable_header
            && !final_config.auth.enable_basic
            && !final_config.auth.disable_all
        {
            tracing::warn!(
                "⚠️  WARNING: No authentication methods are enabled! Access will be denied to all requests."
            );
        }

        Ok(final_config)
    }
}
