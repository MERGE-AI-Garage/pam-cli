//! Configuration management for PAM CLI

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// PAM CLI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// PAM API base URL
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// GCS bucket for context files
    #[serde(default = "default_gcs_bucket")]
    pub gcs_bucket: String,

    /// Default user email
    pub user_email: Option<String>,

    /// Database host
    #[serde(default = "default_db_host")]
    pub db_host: String,

    /// Database port
    #[serde(default = "default_db_port")]
    pub db_port: u16,

    /// Database name
    #[serde(default = "default_db_name")]
    pub db_name: String,

    /// Database user
    #[serde(default = "default_db_user")]
    pub db_user: String,

    /// Database password (prefer env var PAM_DB_PASSWORD)
    pub db_password: Option<String>,

    /// CLI API key for authentication (prefer env var PAM_CLI_API_KEY)
    pub cli_api_key: Option<String>,
}

fn default_api_url() -> String {
    "https://pam-production-service-925072200586.us-central1.run.app".to_string()
}

fn default_gcs_bucket() -> String {
    "pam-context-files".to_string()
}

fn default_db_host() -> String {
    "localhost".to_string()
}

fn default_db_port() -> u16 {
    5433
}

fn default_db_name() -> String {
    "pam_pm_knowledge".to_string()
}

fn default_db_user() -> String {
    "postgres".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: default_api_url(),
            gcs_bucket: default_gcs_bucket(),
            user_email: None,
            db_host: default_db_host(),
            db_port: default_db_port(),
            db_name: default_db_name(),
            db_user: default_db_user(),
            db_password: None,
            cli_api_key: None,
        }
    }
}

impl Config {
    /// Load configuration from file or defaults
    pub fn load(config_path: Option<&str>) -> Result<Self> {
        // Load .env file if present
        let _ = dotenvy::dotenv();

        // Determine config file path
        let path = match config_path {
            Some(p) => PathBuf::from(p),
            None => Self::config_path()?,
        };

        // Load from file if exists
        let mut config = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config file: {}", path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", path.display()))?
        } else {
            Config::default()
        };

        // Override with environment variables
        if let Ok(url) = std::env::var("PAM_API_URL") {
            config.api_url = url;
        }
        if let Ok(bucket) = std::env::var("PAM_GCS_BUCKET") {
            config.gcs_bucket = bucket;
        }
        if let Ok(email) = std::env::var("PAM_USER_EMAIL") {
            config.user_email = Some(email);
        }
        if let Ok(host) = std::env::var("PAM_DB_HOST") {
            config.db_host = host;
        }
        if let Ok(port) = std::env::var("PAM_DB_PORT") {
            config.db_port = port.parse().unwrap_or(5433);
        }
        if let Ok(password) = std::env::var("PAM_DB_PASSWORD") {
            config.db_password = Some(password);
        }

        Ok(config)
    }

    /// Get the default config file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("pam");

        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("config.toml"))
    }

    /// Initialize a new config file
    pub fn init(force: bool) -> Result<()> {
        let path = Self::config_path()?;

        if path.exists() && !force {
            anyhow::bail!(
                "Config file already exists at {}. Use --force to overwrite.",
                path.display()
            );
        }

        let default_config = Config::default();
        let content = toml::to_string_pretty(&default_config)?;

        std::fs::write(&path, content)?;
        println!("Created config file at: {}", path.display());

        Ok(())
    }

    /// Set a configuration value
    pub fn set_value(key: &str, value: &str) -> Result<()> {
        let path = Self::config_path()?;
        let mut config = Self::load(None)?;

        match key {
            "api_url" => config.api_url = value.to_string(),
            "gcs_bucket" => config.gcs_bucket = value.to_string(),
            "user_email" => config.user_email = Some(value.to_string()),
            "db_host" => config.db_host = value.to_string(),
            "db_port" => config.db_port = value.parse()?,
            "db_name" => config.db_name = value.to_string(),
            "db_user" => config.db_user = value.to_string(),
            _ => anyhow::bail!("Unknown config key: {}", key),
        }

        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&path, content)?;

        Ok(())
    }

    /// Get database connection string
    pub fn db_connection_string(&self) -> String {
        let password = self
            .db_password
            .clone()
            .or_else(|| std::env::var("PAM_DB_PASSWORD").ok())
            .unwrap_or_default();

        format!(
            "host={} port={} dbname={} user={} password={}",
            self.db_host, self.db_port, self.db_name, self.db_user, password
        )
    }
}
