use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

use fylge::{create_router, init_pool, run_migrations, AppState, Icon};

/// Create a test app with in-memory database.
async fn create_test_app() -> axum::Router {
    let pool = init_pool("sqlite::memory:").await.unwrap();
    run_migrations(&pool).await.unwrap();

    let icons = vec![
        Icon {
            id: "marker".to_string(),
            name: "Marker".to_string(),
            url: "/static/icons/marker.svg".to_string(),
        },
        Icon {
            id: "ship".to_string(),
            name: "Ship".to_string(),
            url: "/static/icons/ship.svg".to_string(),
        },
    ];

    let state = AppState::new(pool, icons);
    create_router(state)
}

/// Helper to get response body as string.
async fn body_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

// ============================================================================
// Health endpoint tests
// ============================================================================

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_string(response.into_body()).await;
    assert_eq!(body, "OK");
}

// ============================================================================
// Icons endpoint tests
// ============================================================================

#[tokio::test]
async fn test_get_icons() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/icons")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_string(response.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert!(json["icons"].is_array());
    assert_eq!(json["icons"].as_array().unwrap().len(), 2);
    assert_eq!(json["icons"][0]["id"], "marker");
    assert_eq!(json["icons"][1]["id"], "ship");
}

// ============================================================================
// Markers endpoint tests
// ============================================================================

#[tokio::test]
async fn test_get_markers_empty() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/markers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_string(response.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["window_hours"], 24);
    assert!(json["markers"].is_array());
    assert_eq!(json["markers"].as_array().unwrap().len(), 0);
    assert_eq!(json["max_id"], 0);
}

#[tokio::test]
async fn test_create_marker() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/markers")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                        "uuid": "550e8400-e29b-41d4-a716-446655440000",
                        "lat": 59.91,
                        "lon": 10.75,
                        "icon_id": "marker",
                        "label": "Oslo"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_string(response.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["status"], "created");
    assert_eq!(
        json["marker"]["uuid"],
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_eq!(json["marker"]["lat"], 59.91);
    assert_eq!(json["marker"]["lon"], 10.75);
    assert_eq!(json["marker"]["icon_id"], "marker");
    assert_eq!(json["marker"]["label"], "Oslo");
    assert_eq!(json["marker"]["id"], 1);
}

#[tokio::test]
async fn test_create_marker_without_label() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/markers")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                        "uuid": "550e8400-e29b-41d4-a716-446655440000",
                        "lat": 59.91,
                        "lon": 10.75,
                        "icon_id": "marker"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_string(response.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["status"], "created");
    assert!(json["marker"]["label"].is_null());
}

#[tokio::test]
async fn test_create_marker_idempotent() {
    let app = create_test_app().await;

    // First request - create
    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/markers")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                        "uuid": "550e8400-e29b-41d4-a716-446655440000",
                        "lat": 59.91,
                        "lon": 10.75,
                        "icon_id": "marker",
                        "label": "Oslo"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    let body1 = body_string(response1.into_body()).await;
    let json1: serde_json::Value = serde_json::from_str(&body1).unwrap();
    assert_eq!(json1["status"], "created");

    // Second request with same UUID - should return exists
    let response2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/markers")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                        "uuid": "550e8400-e29b-41d4-a716-446655440000",
                        "lat": 60.0,
                        "lon": 11.0,
                        "icon_id": "ship",
                        "label": "Different"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);

    let body2 = body_string(response2.into_body()).await;
    let json2: serde_json::Value = serde_json::from_str(&body2).unwrap();

    assert_eq!(json2["status"], "exists");
    // Original values preserved
    assert_eq!(json2["marker"]["lat"], 59.91);
    assert_eq!(json2["marker"]["icon_id"], "marker");
    assert_eq!(json2["marker"]["label"], "Oslo");
}

#[tokio::test]
async fn test_create_marker_invalid_uuid() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/markers")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                        "uuid": "not-a-valid-uuid",
                        "lat": 59.91,
                        "lon": 10.75,
                        "icon_id": "marker"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_string(response.into_body()).await;
    assert!(body.contains("Invalid UUID"));
}

#[tokio::test]
async fn test_create_marker_invalid_latitude() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/markers")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                        "uuid": "550e8400-e29b-41d4-a716-446655440000",
                        "lat": 91.0,
                        "lon": 10.75,
                        "icon_id": "marker"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_string(response.into_body()).await;
    assert!(body.contains("Invalid latitude"));
}

#[tokio::test]
async fn test_create_marker_invalid_longitude() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/markers")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                        "uuid": "550e8400-e29b-41d4-a716-446655440000",
                        "lat": 59.91,
                        "lon": 181.0,
                        "icon_id": "marker"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_string(response.into_body()).await;
    assert!(body.contains("Invalid longitude"));
}

#[tokio::test]
async fn test_create_marker_empty_icon_id() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/markers")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                        "uuid": "550e8400-e29b-41d4-a716-446655440000",
                        "lat": 59.91,
                        "lon": 10.75,
                        "icon_id": ""
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_string(response.into_body()).await;
    assert!(body.contains("icon_id is required"));
}

// ============================================================================
// Log endpoint tests
// ============================================================================

#[tokio::test]
async fn test_get_log_empty() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/log")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_string(response.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["after_id"], 0);
    assert_eq!(json["limit"], 100);
    assert_eq!(json["max_id"], 0);
    assert_eq!(json["has_more"], false);
    assert!(json["entries"].is_array());
    assert_eq!(json["entries"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_log_with_entries() {
    let app = create_test_app().await;

    // Create a marker first
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/markers")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                        "uuid": "550e8400-e29b-41d4-a716-446655440000",
                        "lat": 59.91,
                        "lon": 10.75,
                        "icon_id": "marker",
                        "label": "Oslo"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Get log
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/log?after_id=0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_string(response.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["entries"].as_array().unwrap().len(), 1);
    assert_eq!(json["max_id"], 1);
    assert_eq!(json["has_more"], false);
}

#[tokio::test]
async fn test_get_log_pagination() {
    let app = create_test_app().await;

    // Create 3 markers
    for i in 1..=3 {
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/markers")
                    .header("Content-Type", "application/json")
                    .body(Body::from(format!(
                        r#"{{
                            "uuid": "550e8400-e29b-41d4-a716-44665544000{}",
                            "lat": 59.91,
                            "lon": 10.75,
                            "icon_id": "marker"
                        }}"#,
                        i
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    // Get first page (limit 2)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/log?after_id=0&limit=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_string(response.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["entries"].as_array().unwrap().len(), 2);
    assert_eq!(json["has_more"], true);
    assert_eq!(json["max_id"], 2);

    // Get second page
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/log?after_id=2&limit=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_string(response.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["entries"].as_array().unwrap().len(), 1);
    assert_eq!(json["has_more"], false);
    assert_eq!(json["max_id"], 3);
}

// ============================================================================
// Integration tests: create and retrieve
// ============================================================================

#[tokio::test]
async fn test_create_and_get_markers() {
    let app = create_test_app().await;

    // Create two markers
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/markers")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                        "uuid": "550e8400-e29b-41d4-a716-446655440001",
                        "lat": 59.91,
                        "lon": 10.75,
                        "icon_id": "marker",
                        "label": "Oslo"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/markers")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                        "uuid": "550e8400-e29b-41d4-a716-446655440002",
                        "lat": 60.39,
                        "lon": 5.32,
                        "icon_id": "ship",
                        "label": "Bergen"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Get all markers
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/markers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_string(response.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    let markers = json["markers"].as_array().unwrap();
    assert_eq!(markers.len(), 2);
    assert_eq!(json["max_id"], 2);

    // Check that both markers are present
    let labels: Vec<&str> = markers
        .iter()
        .map(|m| m["label"].as_str().unwrap())
        .collect();
    assert!(labels.contains(&"Oslo"));
    assert!(labels.contains(&"Bergen"));
}
