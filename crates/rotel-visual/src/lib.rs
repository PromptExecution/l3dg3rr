use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use ledger_core::observability::{
    ClassifiedJournalArtifact, LogShapeClassifier, LogShapeRule, OTelLogRecord,
    OTelSeverityNumber, TelemetryArrowBatch,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, instrument};

#[derive(Serialize, Deserialize, Debug)]
struct HealthResponse {
    status: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TelemetryData {
    logs: Vec<LogRecord>,
    metrics: Vec<MetricRecord>,
    spans: Vec<SpanRecord>,
    classified: Vec<ClassifiedArtifactView>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LogRecord {
    timestamp: String,
    level: String,
    message: String,
    shape: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MetricRecord {
    name: String,
    value: f64,
    timestamp: String,
    labels: Vec<Label>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SpanRecord {
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
    name: String,
    start_time: String,
    end_time: Option<String>,
    status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Label {
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClassifiedArtifactView {
    artifact_id: String,
    signal: String,
    abstract_regex_type: String,
    metric_name: String,
    metric_delta: i64,
    severity_text: String,
    matched_excerpt: String,
    justification_digest: String,
    source_time_unix_nano: u64,
}

impl From<&ClassifiedJournalArtifact> for ClassifiedArtifactView {
    fn from(artifact: &ClassifiedJournalArtifact) -> Self {
        Self {
            artifact_id: artifact.artifact_id.clone(),
            signal: artifact.signal.as_str().to_string(),
            abstract_regex_type: artifact.abstract_regex_type.clone(),
            metric_name: artifact.justification.metric_name.clone(),
            metric_delta: artifact.justification.metric_delta,
            severity_text: artifact.severity_text.clone(),
            matched_excerpt: artifact.matched_excerpt.clone(),
            justification_digest: artifact.justification.evidence_digest.clone(),
            source_time_unix_nano: artifact.source_time_unix_nano,
        }
    }
}

#[derive(Debug)]
struct AppState {
    telemetry_tx: broadcast::Sender<TelemetryData>,
    classifier: LogShapeClassifier,
    ring_buffer: RwLock<VecDeque<TelemetryData>>,
}

impl AppState {
    fn new() -> Result<Self, anyhow::Error> {
        let (tx, _rx) = broadcast::channel(100);
        let classifier = Self::build_classifier()?;
        Ok(Self {
            telemetry_tx: tx,
            classifier,
            ring_buffer: RwLock::new(VecDeque::with_capacity(100)),
        })
    }

    fn build_classifier() -> Result<LogShapeClassifier, anyhow::Error> {
        let rules = vec![
            LogShapeRule {
                rule_id: "gpu-driver-device-disappeared".to_string(),
                abstract_regex_type: "hardware.gpu.driver.device_handle_unknown".to_string(),
                pattern: "Unable to determine the device handle for GPU[0-9]+.*Unknown Error"
                    .to_string(),
                metric_name: "l3dg3rr.hardware.gpu.driver_faults".to_string(),
                metric_delta: 1,
                min_severity: OTelSeverityNumber::Error,
                rationale: "GPU was expected but nvidia-smi returned an unknown device-handle error"
                    .to_string(),
            },
        ];
        Ok(LogShapeClassifier::new(rules)?)
    }

    async fn broadcast(&self, data: TelemetryData) {
        // Push to ring buffer
        {
            let mut buf = self.ring_buffer.write().await;
            if buf.len() >= 100 {
                buf.pop_front();
            }
            buf.push_back(data.clone());
        }
        // Broadcast to subscribers
        let _ = self.telemetry_tx.send(data);
    }

    async fn replay_buffer(&self) -> Vec<TelemetryData> {
        let buf = self.ring_buffer.read().await;
        buf.iter().cloned().collect()
    }
}

#[instrument]
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        message: "Rotel Visual OTel Surface is running".to_string(),
    })
}

#[instrument]
async fn dashboard_handler() -> impl IntoResponse {
    let html = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Rotel OTel Visual Surface</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }
            .dashboard { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
            .panel { border: 1px solid #ddd; padding: 15px; border-radius: 5px; }
            .logs { height: 400px; overflow-y: auto; }
            .metrics { height: 400px; }
            .spans { height: 400px; }
            .classified { height: 400px; overflow-y: auto; }
            .log-entry { margin-bottom: 5px; padding: 5px; border-radius: 3px; }
            .log-entry.error { background-color: #ffe6e6; }
            .log-entry.warn { background-color: #fff3cd; }
            .log-entry.info { background-color: #e7f3ff; }
            .artifact { margin-bottom: 5px; padding: 5px; background-color: #f0f0f0; border-radius: 3px; }
        </style>
    </head>
    <body>
        <h1>Rotel OTel Visual Surface</h1>
        <div class="dashboard">
            <div class="panel">
                <h2>Real-time Logs</h2>
                <div id="logs" class="logs"></div>
            </div>
            <div class="panel">
                <h2>Metrics</h2>
                <div id="metrics" class="metrics"></div>
            </div>
            <div class="panel">
                <h2>Trace Spans</h2>
                <div id="spans" class="spans"></div>
            </div>
            <div class="panel">
                <h2>Classified Artifacts</h2>
                <div id="classified" class="classified"></div>
            </div>
            <div class="panel">
                <h2>System Status</h2>
                <div id="status">Connected</div>
            </div>
        </div>

        <script>
            const logsDiv = document.getElementById('logs');
            const metricsDiv = document.getElementById('metrics');
            const spansDiv = document.getElementById('spans');
            const classifiedDiv = document.getElementById('classified');

            const ws = new WebSocket('ws://' + location.host + '/ws/telemetry');

            ws.onmessage = function(event) {
                const data = JSON.parse(event.data);
                if (Array.isArray(data)) {
                    // Replay buffer
                    data.forEach(batch => updateBatch(batch));
                } else {
                    updateBatch(data);
                }
            };

            function updateBatch(data) {
                if (data.logs) updateLogs(data.logs);
                if (data.metrics) updateMetrics(data.metrics);
                if (data.spans) updateSpans(data.spans);
                if (data.classified) updateClassified(data.classified);
            }

            function updateLogs(logs) {
                logsDiv.innerHTML = logs.map(log => `
                    <div class="log-entry ${log.level.toLowerCase()}">
                        <strong>${log.timestamp}</strong> [${log.level}] ${log.message}
                        <small>(Shape: ${log.shape})</small>
                    </div>
                `).join('');
            }

            function updateMetrics(metrics) {
                metricsDiv.innerHTML = metrics.map(metric => `
                    <div>
                        <strong>${metric.name}</strong>: ${metric.value}
                        <small>${metric.labels.map(l => `${l.key}=${l.value}`).join(', ')}</small>
                    </div>
                `).join('');
            }

            function updateSpans(spans) {
                spansDiv.innerHTML = spans.map(span => `
                    <div>
                        <strong>${span.name}</strong>
                        <small>Trace: ${span.trace_id.substring(0, 8)}...</small>
                        <small>Span: ${span.span_id.substring(0, 8)}...</small>
                        <small>Status: ${span.status}</small>
                    </div>
                `).join('');
            }

            function updateClassified(artifacts) {
                classifiedDiv.innerHTML = artifacts.map(artifact => `
                    <div class="artifact">
                        <strong>${artifact.abstract_regex_type}</strong>
                        <small>Metric: ${artifact.metric_name} (+${artifact.metric_delta})</small>
                        <small>Severity: ${artifact.severity_text}</small>
                        <small>Excerpt: ${artifact.matched_excerpt.substring(0, 60)}...</small>
                    </div>
                `).join('');
            }

            ws.onopen = function() {
                console.log('Connected to telemetry stream');
            };

            ws.onclose = function() {
                console.log('Disconnected from telemetry stream');
                document.getElementById('status').textContent = 'Disconnected';
            };
        </script>
    </body>
    </html>
    "#;

    axum::response::Html(html.to_string())
}

#[instrument]
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // Replay ring buffer first
    let replay = state.replay_buffer().await;
    for batch in replay {
        let msg = axum::extract::ws::Message::Text(
            serde_json::to_string(&batch).unwrap().into()
        );
        if let Err(err) = socket.send(msg).await {
            error!("Error sending replay data: {}", err);
            return;
        }
    }

    // Subscribe to live updates
    let mut rx = state.telemetry_tx.subscribe();

    loop {
        match rx.recv().await {
            Ok(telemetry) => {
                let msg = axum::extract::ws::Message::Text(
                    serde_json::to_string(&telemetry).unwrap().into()
                );
                if let Err(err) = socket.send(msg).await {
                    error!("Error sending telemetry data: {}", err);
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

// OTLP JSON ingestion handlers

#[derive(Serialize, Deserialize, Debug)]
struct OtlpLogsRequest {
    #[serde(rename = "resourceLogs")]
    resource_logs: Vec<ResourceLogs>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ResourceLogs {
    resource: Option<OtlpResource>,
    #[serde(rename = "scopeLogs")]
    scope_logs: Vec<ScopeLogs>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OtlpResource {
    attributes: Vec<OtlpAttribute>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ScopeLogs {
    #[serde(rename = "logRecords")]
    log_records: Vec<OtlpLogRecord>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OtlpLogRecord {
    #[serde(rename = "timeUnixNano")]
    time_unix_nano: String,
    #[serde(rename = "severityNumber")]
    severity_number: u8,
    #[serde(rename = "severityText")]
    severity_text: String,
    body: OtlpAnyValue,
    attributes: Vec<OtlpAttribute>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OtlpAnyValue {
    #[serde(rename = "stringValue")]
    string_value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OtlpAttribute {
    key: String,
    value: OtlpAnyValue,
}

#[derive(Serialize, Deserialize, Debug)]
struct OtlpIngestResponse {
    accepted: bool,
    signal: String,
    resource_count: usize,
    classification_columns: Vec<String>,
}

#[instrument]
async fn otlp_logs_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let request: Result<OtlpLogsRequest, _> = serde_json::from_value(payload.clone());
    match request {
        Ok(req) => {
            let mut logs = Vec::new();
            let mut classified = Vec::new();

            for resource_log in &req.resource_logs {
                for scope_log in &resource_log.scope_logs {
                    for record in &scope_log.log_records {
                        let body = record.body.string_value.clone().unwrap_or_default();
                        let severity = otlp_severity_to_enum(record.severity_number);

                        let log = OTelLogRecord::new(
                            record.time_unix_nano.parse().unwrap_or(0),
                            severity,
                            body.clone(),
                        );

                        // Classify the log
                        let artifacts = state.classifier.classify_log(&log);
                        for artifact in &artifacts {
                            classified.push(ClassifiedArtifactView::from(artifact));
                        }

                        logs.push(LogRecord {
                            timestamp: record.time_unix_nano.clone(),
                            level: record.severity_text.clone(),
                            message: body,
                            shape: if artifacts.is_empty() {
                                "unclassified".to_string()
                            } else {
                                artifacts[0].abstract_regex_type.clone()
                            },
                        });
                    }
                }
            }

            let telemetry = TelemetryData {
                logs,
                metrics: vec![],
                spans: vec![],
                classified,
            };

            state.broadcast(telemetry).await;

            let response = Json(OtlpIngestResponse {
                accepted: true,
                signal: "log".to_string(),
                resource_count: req.resource_logs.len(),
                classification_columns: TelemetryArrowBatch::classification_columns()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            });
            (axum::http::StatusCode::ACCEPTED, response).into_response()
        }
        Err(error) => {
            error!("Invalid OTLP JSON payload: {}", error);
            let response = Json(serde_json::json!({
                "error": format!("invalid OTLP JSON payload: {error}")
            }));
            return (axum::http::StatusCode::BAD_REQUEST, response).into_response();
        }
    }
}

fn otlp_severity_to_enum(severity_number: u8) -> OTelSeverityNumber {
    match severity_number {
        1..=4 => OTelSeverityNumber::Trace,
        5..=8 => OTelSeverityNumber::Debug,
        9..=12 => OTelSeverityNumber::Info,
        13..=16 => OTelSeverityNumber::Warn,
        17..=20 => OTelSeverityNumber::Error,
        _ => OTelSeverityNumber::Fatal,
    }
}

#[instrument]
async fn otlp_metrics_handler(
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let resource_count = payload
        .get("resourceMetrics")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let response = Json(OtlpIngestResponse {
        accepted: true,
        signal: "metric".to_string(),
        resource_count,
        classification_columns: TelemetryArrowBatch::classification_columns()
            .iter()
            .map(|s| s.to_string())
            .collect(),
    });
    (axum::http::StatusCode::ACCEPTED, response).into_response()
}

#[instrument]
async fn otlp_traces_handler(
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let resource_count = payload
        .get("resourceSpans")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let response = Json(OtlpIngestResponse {
        accepted: true,
        signal: "trace".to_string(),
        resource_count,
        classification_columns: TelemetryArrowBatch::classification_columns()
            .iter()
            .map(|s| s.to_string())
            .collect(),
    });
    (axum::http::StatusCode::ACCEPTED, response).into_response()
}

pub fn create_app() -> Router {
    let state = Arc::new(AppState::new().expect("Failed to initialize app state"));

    Router::new()
        .route("/", get(dashboard_handler))
        .route("/health", get(health_handler))
        .route("/ws/telemetry", get(websocket_handler))
        .route("/v1/logs", post(otlp_logs_handler))
        .route("/v1/metrics", post(otlp_metrics_handler))
        .route("/v1/traces", post(otlp_traces_handler))
        .with_state(state)
}

pub async fn run_server() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    info!("Rotel Visual OTel Surface starting on 0.0.0.0:8080");

    axum::serve(listener, create_app()).await.unwrap();
}
