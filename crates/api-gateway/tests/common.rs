use axum::{
    http::Method,
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
};

/// Root endpoint for testing
async fn root() -> &'static str {
    "api gateway: okay"
}

/// Health check endpoint for testing
async fn health() -> &'static str {
    "ok"
}

/// Create a test app with the same middleware stack as the main app
pub fn create_test_app() -> Router {
    // Configure CORS middleware (same as main app)
    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderName::from_static("x-request-id"),
        ])
        .expose_headers([axum::http::HeaderName::from_static("x-request-id")]);

    // Build HTTP router with middleware (same as main app)
    Router::new()
        .route("/", get(root))
        .route("/healthz", get(health))
        .layer(axum::middleware::from_fn(api_gateway::request_id_middleware))
        .layer(
            ServiceBuilder::new()
                .layer(cors_layer)
        )
}

// The request_id_middleware is now accessible via api_gateway::request_id_middleware
