pub mod config;

use axum::{
    extract::Request,
    http::HeaderName,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Request ID middleware that ensures every request has a unique x-request-id header
/// 
/// - Preserves client-provided x-request-id if present
/// - Generates new UUIDv4 if missing
/// - Stores ID in request extensions for downstream access
/// - Adds ID to response headers
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    // Get or generate request ID
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|header| header.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Store in request extensions for downstream access
    request.extensions_mut().insert(request_id.clone());

    // Log the request ID for tracing
    tracing::info!("Processing request with ID: {}", request_id);

    // Process the request
    let mut response = next.run(request).await;

    // Add x-request-id to response headers
    response.headers_mut().insert(
        HeaderName::from_static("x-request-id"),
        request_id.parse().unwrap(),
    );

    response
}