mod config;

use crate::config::AppConfig;
use axum::{
    http::Method,
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Root endpoint - returns service status
async fn root() -> &'static str {
    "api gateway: okay"
}

/// Health check endpoint for monitoring and load balancers
async fn health() -> &'static str {
    "ok"
}

// ============================================================================
// Application Setup
// ============================================================================

/// Main entry point for the API Gateway service
/// 
/// Initializes logging, loads configuration, sets up middleware, and starts the server.
/// Supports hierarchical configuration: defaults < config.toml < environment variables.
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load and validate configuration
    let cfg = AppConfig::load().map_err(|e| anyhow::anyhow!("Config error: {}", e))?;
    tracing::info!(?cfg, "loaded config");
    
    let addr = cfg.addr();

    // Configure CORS middleware
    let cors_layer = if cfg.cors_origins.contains(&"*".to_string()) {
        // Allow all origins (development mode)
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    } else {
        // Validate specific origins
        let origins: Result<Vec<_>, _> = cfg.cors_origins.iter()
            .map(|origin| origin.parse())
            .collect();
        CorsLayer::new()
            .allow_origin(origins.map_err(|e| anyhow::anyhow!("Invalid CORS origin: {}", e))?)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    };

    // Build HTTP router with middleware
    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(health))
        .layer(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(cfg.timeout_duration()))
                .layer(cors_layer)
        );

    // Start server
    let listener = TcpListener::bind(&addr).await?;
    
    tracing::info!("listening on http://{}", listener.local_addr()?);
    tracing::info!("CORS origins: {:?}", cfg.cors_origins);
    tracing::info!("Request timeout: {}ms", cfg.request_timeout_ms);
    tracing::info!("Upstream services: {:?}", cfg.upstreams);
    
    axum::serve(listener, app).await?;
    Ok(())
}