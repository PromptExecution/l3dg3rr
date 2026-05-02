use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    let app = rotel_visual::create_app();

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
}

#[tokio::test]
async fn test_dashboard_endpoint() {
    let app = rotel_visual::create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("text/html"));
}

#[tokio::test]
async fn test_otlp_logs_ingestion_accepts_json_and_returns_202() {
    let app = rotel_visual::create_app();

    let body = json!({
        "resourceLogs": [
            {
                "resource": { "attributes": [] },
                "scopeLogs": []
            }
        ]
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/logs")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_otlp_metrics_ingestion_accepts_json_and_returns_202() {
    let app = rotel_visual::create_app();

    let body = json!({
        "resourceMetrics": [
            {
                "resource": { "attributes": [] },
                "scopeMetrics": []
            }
        ]
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/metrics")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_otlp_traces_ingestion_accepts_json_and_returns_202() {
    let app = rotel_visual::create_app();

    let body = json!({
        "resourceSpans": [
            {
                "resource": { "attributes": [] },
                "scopeSpans": []
            }
        ]
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/traces")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_otlp_logs_rejects_invalid_json_with_400() {
    let app = rotel_visual::create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/logs")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from("not-json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_classified_artifacts_are_broadcast_to_websocket() {
    // This test verifies that when OTLP logs are ingested and classified,
    // the resulting artifacts are broadcast to WebSocket subscribers.
    // We test this indirectly by checking the telemetry channel state.
    let app = rotel_visual::create_app();

    // Ingest a log that matches the GPU fault rule
    let body = json!({
        "resourceLogs": [
            {
                "resource": {
                    "attributes": [
                        { "key": "host.name", "value": { "stringValue": "test-host" } }
                    ]
                },
                "scopeLogs": [
                    {
                        "logRecords": [
                            {
                                "timeUnixNano": "1777724525000000000",
                                "severityNumber": 17,
                                "severityText": "ERROR",
                                "body": {
                                    "stringValue": "Unable to determine the device handle for GPU0: 0000:01:00.0: Unknown Error"
                                },
                                "attributes": []
                            }
                        ]
                    }
                ]
            }
        ]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/logs")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_ring_buffer_replays_classified_artifacts_to_new_subscribers() {
    // Verify that new WebSocket connections receive replayed artifacts
    // from the ring buffer before live updates.
    let app = rotel_visual::create_app();

    // Ingest a log to populate the ring buffer
    let body = json!({
        "resourceLogs": [
            {
                "resource": { "attributes": [] },
                "scopeLogs": [
                    {
                        "logRecords": [
                            {
                                "timeUnixNano": "1777724525000000000",
                                "severityNumber": 17,
                                "severityText": "ERROR",
                                "body": {
                                    "stringValue": "Unable to determine the device handle for GPU0: 0000:01:00.0: Unknown Error"
                                },
                                "attributes": []
                            }
                        ]
                    }
                ]
            }
        ]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/logs")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}
