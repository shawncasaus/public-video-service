//! Configuration integration tests
//! 
//! Tests configuration precedence hierarchy: defaults < config file < environment variables
//! Uses test-specific config files to prevent interference between tests.

use api_gateway::config::AppConfig;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::thread;

// ============================================================================
// Test Utilities
// ============================================================================

fn cleanup_env() {
    env::remove_var("APP_HOST");
    env::remove_var("APP_PORT");
    env::remove_var("APP_REQUEST_TIMEOUT_MS");
    env::remove_var("APP_CORS_ORIGINS");
    env::remove_var("APP_UPSTREAMS__ENV_SERVICE");
    env::remove_var("APP_UPSTREAMS__INVALID_SERVICE");
}

fn get_test_config_path() -> String {
    let thread_id = thread::current().id();
    format!("config_test_{:?}.toml", thread_id)
}

fn create_test_config(content: &str) -> String {
    let config_path = get_test_config_path();
    fs::write(&config_path, content).expect("Failed to write test config file");
    config_path
}

fn cleanup_test_config(config_path: &str) {
    if Path::new(config_path).exists() {
        fs::remove_file(config_path).expect("Failed to remove test config file");
    }
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_config_defaults_only() {
    cleanup_env();
    
    // Ensure no config file exists
    let config_path = "../../config.toml";
    if Path::new(config_path).exists() {
        fs::remove_file(config_path).expect("Failed to remove config file");
    }
    
    let config = AppConfig::load().expect("Failed to load config");
    
    // Verify default values
    assert_eq!(config.host, "");
    assert_eq!(config.port, 3000);
    assert_eq!(config.request_timeout_ms, 15000);
    assert_eq!(config.cors_origins, vec!["*"]);
    assert!(config.upstreams.is_empty());
}

#[test]
fn test_config_file_overrides_defaults() {
    cleanup_env();
    
    let config_content = r#"
host = "file.example.com"
port = 4000
request_timeout_ms = 20000
cors_origins = ["https://file.example.com"]

[upstreams]
file_service = "http://file-service:3001"
"#;
    
    let test_config_path = create_test_config(config_content);
    let config = AppConfig::load_from_file(&test_config_path).expect("Failed to load config");
    
    // Verify file values override defaults
    assert_eq!(config.host, "file.example.com");
    assert_eq!(config.port, 4000);
    assert_eq!(config.request_timeout_ms, 20000);
    assert_eq!(config.cors_origins, vec!["https://file.example.com"]);
    
    let mut expected_upstreams = HashMap::new();
    expected_upstreams.insert("file_service".to_string(), "http://file-service:3001".to_string());
    assert_eq!(config.upstreams, expected_upstreams);
    
    cleanup_test_config(&test_config_path);
}

#[test]
fn test_config_validation_errors() {
    cleanup_env();
    
    // Ensure no config file exists
    let config_path = "../../config.toml";
    if Path::new(config_path).exists() {
        fs::remove_file(config_path).expect("Failed to remove config file");
    }
    
    // Test invalid port number (0 is not allowed)
    env::set_var("APP_PORT", "0");
    let result = AppConfig::load();
    assert!(result.is_err(), "Expected error for invalid port 0");
    assert!(result.unwrap_err().to_string().contains("Invalid port"));
    env::remove_var("APP_PORT");
    
    // Test invalid timeout (0ms is not allowed)
    env::set_var("APP_REQUEST_TIMEOUT_MS", "0");
    let result = AppConfig::load();
    match result {
        Ok(config) => {
            println!("Config loaded successfully with timeout: {}", config.request_timeout_ms);
            panic!("Expected error for invalid timeout 0ms, but got: {}", config.request_timeout_ms);
        },
        Err(e) => {
            println!("Got expected error: {}", e);
            assert!(e.to_string().contains("Invalid timeout"));
        }
    }
    env::remove_var("APP_REQUEST_TIMEOUT_MS");
    
    // Test invalid upstream URL format
    env::set_var("APP_UPSTREAMS__INVALID_SERVICE", "not-a-url");
    let result = AppConfig::load();
    assert!(result.is_err(), "Expected error for invalid upstream URL");
    assert!(result.unwrap_err().to_string().contains("Invalid upstream URL"));
    env::remove_var("APP_UPSTREAMS__INVALID_SERVICE");
}

#[test]
fn test_config_file_validation_errors() {
    cleanup_env();
    
    let invalid_config_content = r#"
host = "file.example.com"
port = 0
request_timeout_ms = 0
cors_origins = [""]

[upstreams]
invalid_service = "not-a-url"
"#;
    
    let test_config_path = create_test_config(invalid_config_content);
    let result = AppConfig::load_from_file(&test_config_path);
    
    assert!(result.is_err(), "Expected validation error for invalid config file");
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    
    assert!(error_msg.contains("Invalid") || error_msg.contains("invalid"), 
            "Error message should indicate validation failure: {}", error_msg);
    
    cleanup_test_config(&test_config_path);
}