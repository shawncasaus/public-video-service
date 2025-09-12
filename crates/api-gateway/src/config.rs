use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use url::Url;

/// Application configuration for the API Gateway service.
/// 
/// Supports hierarchical configuration loading with precedence:
/// defaults < config file < environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server bind address (empty = all interfaces)
    #[serde(default = "default_host")]
    pub host: String,
    
    /// Server port (1-65535)
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// Request timeout in milliseconds (1-300000)
    #[serde(default = "default_timeout_ms")]
    pub request_timeout_ms: u64,
    
    /// Upstream service mappings (service_name -> URL)
    #[serde(default = "default_upstreams")]
    pub upstreams: HashMap<String, String>,
    
    /// Allowed CORS origins (use ["*"] for all)
    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,
}

/// Raw configuration for deserialization before validation
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AppConfigRaw {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default = "default_upstreams")]
    pub upstreams: HashMap<String, String>,
    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,
}

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file or environment parsing error
    #[error(transparent)]
    Config(#[from] ::config::ConfigError),
    
    /// Generic configuration error with message
    #[error("Configuration error: {0}")]
    Message(String),
    
    /// Port number validation error (must be 1-65535)
    #[error("Invalid port number: {0}. Must be between 1 and 65535")]
    InvalidPort(u16),
    
    /// Request timeout validation error (must be 1-300000ms)
    #[error("Invalid timeout: {0}ms. Must be between 1 and 300000ms (5 minutes)")]
    InvalidTimeout(u64),
    
    /// Upstream URL validation error
    #[error("Invalid upstream URL for service '{0}': {1}")]
    InvalidUpstreamUrl(String, String),
    
    /// CORS origin validation error
    #[error("Invalid CORS origin: {0}")]
    InvalidCorsOrigin(String),
}

// ============================================================================
// Default Values
// ============================================================================

fn default_host() -> String {
    "".into()
}

fn default_port() -> u16 {
    3000
}

fn default_timeout_ms() -> u64 {
    15000
}

fn default_upstreams() -> HashMap<String, String> {
    HashMap::new()
}

fn default_cors_origins() -> Vec<String> {
    vec!["*".to_string()]
}

// ============================================================================
// Configuration Loading
// ============================================================================

impl AppConfig {
    /// Load configuration with precedence: defaults < file < environment variables
    /// 
    /// # Returns
    /// - `Ok(AppConfig)` - Successfully loaded and validated configuration
    /// - `Err(ConfigError)` - Configuration loading or validation failed
    pub fn load() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();
        
        let cfg = ::config::Config::builder()
            .set_default("host", default_host())?
            .set_default("port", default_port())?
            .set_default("request_timeout_ms", default_timeout_ms())?
            .set_default("upstreams", default_upstreams())?
            .set_default("cors_origins", default_cors_origins())?
            .add_source(::config::File::with_name("config").required(false))
            .add_source(::config::File::with_name("../../config").required(false))
            .add_source(::config::Environment::with_prefix("APP").separator("_"))
            .build()?;

        let raw_config: AppConfigRaw = cfg.try_deserialize()?;
        Self::validate_and_convert(raw_config)
    }

    /// Load configuration from a specific file path (primarily for testing)
    /// 
    /// # Arguments
    /// - `config_path` - Path to the configuration file
    /// 
    /// # Returns
    /// - `Ok(AppConfig)` - Successfully loaded and validated configuration
    /// - `Err(ConfigError)` - Configuration loading or validation failed
    pub fn load_from_file(config_path: &str) -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();
        
        let cfg = ::config::Config::builder()
            .set_default("host", default_host())?
            .set_default("port", default_port())?
            .set_default("request_timeout_ms", default_timeout_ms())?
            .set_default("upstreams", default_upstreams())?
            .set_default("cors_origins", default_cors_origins())?
            .add_source(::config::File::with_name(config_path).required(false))
            .add_source(::config::Environment::with_prefix("APP").separator("_"))
            .build()?;

        let raw_config: AppConfigRaw = cfg.try_deserialize()?;
        Self::validate_and_convert(raw_config)
    }

    /// Validate raw configuration and convert to validated AppConfig
    fn validate_and_convert(raw: AppConfigRaw) -> Result<Self, ConfigError> {
        // Validate port number
        if raw.port == 0 {
            return Err(ConfigError::InvalidPort(raw.port));
        }

        // Validate timeout
        if raw.request_timeout_ms == 0 || raw.request_timeout_ms > 300000 {
            return Err(ConfigError::InvalidTimeout(raw.request_timeout_ms));
        }

        // Validate upstream URLs
        for (service_name, url_str) in &raw.upstreams {
            if let Err(e) = Url::parse(url_str) {
                return Err(ConfigError::InvalidUpstreamUrl(
                    service_name.clone(),
                    format!("Invalid URL format: {}", e)
                ));
            }
            
            // Check for valid scheme (http/https)
            if let Ok(url) = Url::parse(url_str) {
                if !matches!(url.scheme(), "http" | "https") {
                    return Err(ConfigError::InvalidUpstreamUrl(
                        service_name.clone(),
                        "URL must use http or https scheme".to_string()
                    ));
                }
            }
        }

        // Validate CORS origins
        for origin in &raw.cors_origins {
            if origin.is_empty() {
                return Err(ConfigError::InvalidCorsOrigin(
                    "CORS origin cannot be empty".to_string()
                ));
            }
            
            // Allow "*" or validate as URL
            if origin != "*" {
                if let Err(e) = Url::parse(origin) {
                    return Err(ConfigError::InvalidCorsOrigin(
                        format!("Invalid origin URL: {}", e)
                    ));
                }
            }
        }

        Ok(AppConfig {
            host: raw.host,
            port: raw.port,
            request_timeout_ms: raw.request_timeout_ms,
            upstreams: raw.upstreams,
            cors_origins: raw.cors_origins,
        })
    }
}

// ============================================================================
// Utility Methods
// ============================================================================

impl AppConfig {
    /// Get server address in "host:port" format
    /// 
    /// Returns "0.0.0.0:port" when host is empty (bind to all interfaces)
    pub fn addr(&self) -> String {
        if self.host.is_empty() {
            format!("0.0.0.0:{}", self.port)
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }

    /// Get request timeout as Duration
    pub fn timeout_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.request_timeout_ms)
    }

    /// Get upstream URL for a service name
    /// 
    /// # Arguments
    /// - `service_name` - Name of the upstream service
    /// 
    /// # Returns
    /// - `Some(&String)` - URL of the service if found
    /// - `None` - Service not configured
    pub fn get_upstream_url(&self, service_name: &str) -> Option<&String> {
        self.upstreams.get(service_name)
    }
}