use api_gateway::config::AppConfig;
use api_gateway::request_id_middleware;
use axum::{
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, DefaultOnFailure};
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

/// Test endpoint that simulates a slow response for timeout testing
async fn slow_endpoint() -> Result<&'static str, ServiceError> {
    tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
    Ok("This should never be reached due to timeout")
}

/// Wrapper function that applies timeout to any async function
async fn with_timeout<F, T>(duration: std::time::Duration, future: F) -> Result<T, ServiceError>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| ServiceError::Timeout(tower::timeout::error::Elapsed::new()))
}

// ============================================================================
// Error Handling
// ============================================================================

/// Custom error type for handling various service errors
#[derive(Debug)]
pub enum ServiceError {
    Timeout(tower::timeout::error::Elapsed),
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        match self {
            ServiceError::Timeout(err) => {
                tracing::warn!("Request timed out: {}", err);

                let error_response = json!({
                    "error": "Gateway Timeout",
                    "message": "The request timed out",
                    "status": 504
                });

                (StatusCode::GATEWAY_TIMEOUT, Json(error_response)).into_response()
            }
            ServiceError::Other(err) => {
                tracing::error!("Service error: {}", err);

                let error_response = json!({
                    "error": "Internal Server Error",
                    "message": "An internal error occurred",
                    "status": 500
                });

                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
            }
        }
    }
}

impl From<tower::timeout::error::Elapsed> for ServiceError {
    fn from(err: tower::timeout::error::Elapsed) -> Self {
        ServiceError::Timeout(err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ServiceError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        ServiceError::Other(err)
    }
}

// ============================================================================
// Trace Middleware
// ============================================================================

// TraceLayer will be applied directly in the router setup

// ============================================================================
// Application Setup
// ============================================================================

/// Main entry point for the API Gateway service
///
/// Initializes logging, loads configuration, sets up middleware, and starts the server.
/// Supports hierarchical configuration: defaults < config.toml < environment variables.
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize structured logging with better formatting
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("tower_http::trace=info".parse().unwrap())
                .add_directive("api_gateway=info".parse().unwrap())
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
        )
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
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
                axum::http::HeaderName::from_static("x-request-id"),
            ])
            .expose_headers([axum::http::HeaderName::from_static("x-request-id")])
    } else {
        // Validate specific origins
        let origins: Result<Vec<_>, _> = cfg
            .cors_origins
            .iter()
            .map(|origin| origin.parse())
            .collect();
        CorsLayer::new()
            .allow_origin(origins.map_err(|e| anyhow::anyhow!("Invalid CORS origin: {}", e))?)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
                axum::http::HeaderName::from_static("x-request-id"),
            ])
            .expose_headers([axum::http::HeaderName::from_static("x-request-id")])
    };

    // Build HTTP router with middleware
    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(health))
        .route(
            "/slow",
            get({
                let timeout_duration = cfg.timeout_duration();
                move || async move { with_timeout(timeout_duration, slow_endpoint()).await }
            }),
        )
        .layer(axum::middleware::from_fn(request_id_middleware))
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .include_headers(true)
                        .level(tracing::Level::INFO)
                )
                .on_request(
                    DefaultOnRequest::new()
                        .level(tracing::Level::INFO)
                )
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                )
                .on_failure(
                    DefaultOnFailure::new()
                        .level(tracing::Level::ERROR)
                )
        )
        .layer(ServiceBuilder::new().layer(cors_layer));

    // Start server
    let listener = TcpListener::bind(&addr).await?;
    let actual_addr = listener.local_addr()?;

    tracing::info!("🚀 API Gateway started successfully");
    tracing::info!("📍 Listening on: http://{}", actual_addr);
    tracing::info!("🔧 Host binding: {} ({})", 
        if cfg.host.is_empty() { "all interfaces (0.0.0.0)" } else { &cfg.host },
        if cfg.host.is_empty() { "external access enabled" } else { "localhost only" }
    );
    tracing::info!("⏱️  Request timeout: {}ms", cfg.request_timeout_ms);
    tracing::info!("🌐 CORS origins: {:?}", cfg.cors_origins);
    tracing::info!("🔗 Upstream services: {:?}", cfg.upstreams);

    axum::serve(listener, app).await?;
    Ok(())
}
