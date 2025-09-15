use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;

mod common;

/// Test that requests without x-request-id header get a generated UUID
#[tokio::test]
async fn test_no_header_generates_uuid() {
    let app = common::create_test_app();

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    // Should have x-request-id header
    let request_id_header = response.headers().get("x-request-id");
    assert!(
        request_id_header.is_some(),
        "Response should include x-request-id header"
    );

    // Should be a valid UUID
    let request_id = request_id_header.unwrap().to_str().unwrap();
    let uuid = Uuid::parse_str(request_id);
    assert!(
        uuid.is_ok(),
        "Request ID should be a valid UUID: {}",
        request_id
    );
}

/// Test that requests with x-request-id header preserve the provided ID
#[tokio::test]
async fn test_with_header_preserves_id() {
    let app = common::create_test_app();

    let provided_id = "test-request-id-12345";

    let request = Request::builder()
        .uri("/")
        .header("x-request-id", provided_id)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    // Should have x-request-id header
    let request_id_header = response.headers().get("x-request-id");
    assert!(
        request_id_header.is_some(),
        "Response should include x-request-id header"
    );

    // Should preserve the provided ID
    let request_id = request_id_header.unwrap().to_str().unwrap();
    assert_eq!(request_id, provided_id, "Request ID should be preserved");
}

/// Test that health endpoint also gets request ID
#[tokio::test]
async fn test_health_endpoint_gets_request_id() {
    let app = common::create_test_app();

    let request = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    // Should have x-request-id header
    let request_id_header = response.headers().get("x-request-id");
    assert!(
        request_id_header.is_some(),
        "Health endpoint should include x-request-id header"
    );

    // Should be a valid UUID
    let request_id = request_id_header.unwrap().to_str().unwrap();
    let uuid = Uuid::parse_str(request_id);
    assert!(
        uuid.is_ok(),
        "Request ID should be a valid UUID: {}",
        request_id
    );
}

/// Test that CORS exposes x-request-id header
#[tokio::test]
async fn test_cors_exposes_request_id() {
    let app = common::create_test_app();

    // Test with a regular GET request to see if CORS headers are exposed
    let request = Request::builder()
        .uri("/")
        .method("GET")
        .header("Origin", "http://localhost:3000")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    // Should have Access-Control-Expose-Headers with x-request-id
    let expose_headers = response.headers().get("access-control-expose-headers");
    assert!(expose_headers.is_some(), "Should expose headers for CORS");

    let expose_headers_str = expose_headers.unwrap().to_str().unwrap();
    assert!(
        expose_headers_str.contains("x-request-id"),
        "Should expose x-request-id header: {}",
        expose_headers_str
    );
}

/// Test that different requests get different UUIDs
#[tokio::test]
async fn test_different_requests_get_different_uuids() {
    let app = common::create_test_app();

    let request1 = Request::builder().uri("/").body(Body::empty()).unwrap();

    let request2 = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();
    let response2 = app.oneshot(request2).await.unwrap();

    let id1 = response1
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();
    let id2 = response2
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();

    assert_ne!(
        id1, id2,
        "Different requests should get different request IDs"
    );
}
