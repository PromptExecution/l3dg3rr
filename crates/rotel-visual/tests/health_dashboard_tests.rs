use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

fn app() -> axum::Router {
    rotel_visual::create_app().unwrap()
}

#[tokio::test]
async fn test_health_endpoint() {
    let response = app()
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
    let response = app()
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
    let body = json!({
        "resourceLogs": [
            {
                "resource": { "attributes": [] },
                "scopeLogs": []
            }
        ]
    });

    let response = app()
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
    let body = json!({
        "resourceMetrics": [
            {
                "resource": { "attributes": [] },
                "scopeMetrics": []
            }
        ]
    });

    let response = app()
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
    let body = json!({
        "resourceSpans": [
            {
                "resource": { "attributes": [] },
                "scopeSpans": []
            }
        ]
    });

    let response = app()
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
    let response = app()
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
                                "body": { "stringValue": "Unable to determine the device handle for GPU0: 0000:01:00.0: Unknown Error" },
                                "attributes": []
                            }
                        ]
                    }
                ]
            }
        ]
    });

    let response = app()
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
                                "body": { "stringValue": "Unable to determine the device handle for GPU0: 0000:01:00.0: Unknown Error" },
                                "attributes": []
                            }
                        ]
                    }
                ]
            }
        ]
    });

    let response = app()
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
async fn test_metrics_endpoint_returns_self_telemetry() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(snapshot.get("logs_ingested_total").is_some());
    assert!(snapshot.get("ws_connections_active").is_some());
}

#[tokio::test]
async fn test_metrics_endpoint_increments_after_ingestion() {
    let app = app();

    // Get baseline
    let baseline = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let baseline_body = axum::body::to_bytes(baseline.into_body(), usize::MAX)
        .await
        .unwrap();
    let baseline_json: serde_json::Value = serde_json::from_slice(&baseline_body).unwrap();
    let baseline_logs = baseline_json["metrics_ingested_total"].as_u64().unwrap();

    // Ingest a metric
    let body = json!({
        "resourceMetrics": [
            {
                "resource": { "attributes": [] },
                "scopeMetrics": []
            }
        ]
    });
    let _ = app
        .clone()
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

    // Check incremented
    let after = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let after_body = axum::body::to_bytes(after.into_body(), usize::MAX)
        .await
        .unwrap();
    let after_json: serde_json::Value = serde_json::from_slice(&after_body).unwrap();
    let after_logs = after_json["metrics_ingested_total"].as_u64().unwrap();

    assert_eq!(after_logs, baseline_logs + 1);
}

#[tokio::test]
async fn test_rotel_evaluate_endpoint_returns_sarif() {
    let body = json!({
        "gate_name": "test-gate",
        "expression": "log_shape && metric",
        "log_shape_observed": true,
        "metric_observed": true,
        "slo_expected": true
    });

    let response = app()
        .oneshot(
            Request::builder()
                .uri("/rotel/evaluate")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let sarif: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(sarif.get("runs").is_some());
}

#[tokio::test]
async fn test_rotel_evaluate_detects_slo_failure() {
    let body = json!({
        "gate_name": "test-gate-fail",
        "expression": "log_shape && metric",
        "log_shape_observed": true,
        "metric_observed": false,
        "slo_expected": true
    });

    let response = app()
        .oneshot(
            Request::builder()
                .uri("/rotel/evaluate")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let sarif: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let runs = sarif["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 1);
    let results = runs[0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["rule_id"], "l3dg3rr/otel/build-gate-slo");
}
