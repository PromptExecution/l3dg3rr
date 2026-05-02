use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use b00t_iface::metric::MetricRegistry;
use b00t_iface::sarif::check_otel_logic_slo_as_sarif;
use ledger_core::observability::{
    otlp_json, ClassifiedJournalArtifact, LogShapeClassifier, OTelLogRecord,
    OTelSeverityNumber, TelemetryArrowBatch,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
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

/// Self-telemetry counters for the rotel-visual surface.
#[derive(Debug, Default)]
struct SurfaceMetrics {
    logs_ingested_total: AtomicU64,
    logs_classified_total: AtomicU64,
    metrics_ingested_total: AtomicU64,
    traces_ingested_total: AtomicU64,
    ws_connections_total: AtomicU64,
    ws_connections_active: AtomicU64,
}

impl SurfaceMetrics {
    fn inc_logs_ingested(&self, n: u64) {
        self.logs_ingested_total.fetch_add(n, Ordering::Relaxed);
    }
    fn inc_logs_classified(&self, n: u64) {
        self.logs_classified_total.fetch_add(n, Ordering::Relaxed);
    }
    fn inc_metrics_ingested(&self, n: u64) {
        self.metrics_ingested_total.fetch_add(n, Ordering::Relaxed);
    }
    fn inc_traces_ingested(&self, n: u64) {
        self.traces_ingested_total.fetch_add(n, Ordering::Relaxed);
    }
    fn inc_ws_connection(&self) {
        self.ws_connections_total.fetch_add(1, Ordering::Relaxed);
        self.ws_connections_active.fetch_add(1, Ordering::Relaxed);
    }
    fn dec_ws_connection(&self) {
        self.ws_connections_active.fetch_sub(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> SurfaceMetricsSnapshot {
        SurfaceMetricsSnapshot {
            logs_ingested_total: self.logs_ingested_total.load(Ordering::Relaxed),
            logs_classified_total: self.logs_classified_total.load(Ordering::Relaxed),
            metrics_ingested_total: self.metrics_ingested_total.load(Ordering::Relaxed),
            traces_ingested_total: self.traces_ingested_total.load(Ordering::Relaxed),
            ws_connections_total: self.ws_connections_total.load(Ordering::Relaxed),
            ws_connections_active: self.ws_connections_active.load(Ordering::Relaxed),
        }
    }
}

#[derive(Serialize, Debug)]
struct SurfaceMetricsSnapshot {
    logs_ingested_total: u64,
    logs_classified_total: u64,
    metrics_ingested_total: u64,
    traces_ingested_total: u64,
    ws_connections_total: u64,
    ws_connections_active: u64,
}

#[derive(Debug)]
struct AppState {
    telemetry_tx: broadcast::Sender<TelemetryData>,
    classifier: LogShapeClassifier,
    ring_buffer: RwLock<VecDeque<TelemetryData>>,
    metrics: SurfaceMetrics,
}

impl AppState {
    fn new() -> Result<Self, anyhow::Error> {
        let (tx, _rx) = broadcast::channel(100);
        let classifier = LogShapeClassifier::with_builtin_rules()?;
        Ok(Self {
            telemetry_tx: tx,
            classifier,
            ring_buffer: RwLock::new(VecDeque::with_capacity(100)),
            metrics: SurfaceMetrics::default(),
        })
    }

    async fn broadcast(&self, data: TelemetryData) {
        {
            let mut buf = self.ring_buffer.write().await;
            if buf.len() >= 100 {
                buf.pop_front();
            }
            buf.push_back(data.clone());
        }
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
async fn metrics_handler(State(state): State<Arc<AppState>>) -> Json<SurfaceMetricsSnapshot> {
    Json(state.metrics.snapshot())
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
    state.metrics.inc_ws_connection();

    let replay = state.replay_buffer().await;
    for batch in replay {
        let msg = axum::extract::ws::Message::Text(
            serde_json::to_string(&batch).unwrap().into()
        );
        if let Err(err) = socket.send(msg).await {
            error!("Error sending replay data: {}", err);
            state.metrics.dec_ws_connection();
            return;
        }
    }

    let mut rx = state.telemetry_tx.subscribe();

    loop {
        tokio::select! {
            ws_msg = socket.recv() => {
                match ws_msg {
                    Some(Ok(axum::extract::ws::Message::Close(_))) => break,
                    Some(Ok(_)) => continue,
                    Some(Err(err)) => {
                        error!("Error receiving websocket message: {}", err);
                        break;
                    }
                    None => break,
                }
            }
            recv = rx.recv() => {
                match recv {
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
    }

    state.metrics.dec_ws_connection();
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
    let request: Result<otlp_json::LogsRequest, _> = serde_json::from_value(payload.clone());
    match request {
        Ok(req) => {
            let mut logs = Vec::new();
            let mut classified = Vec::new();
            let mut log_count = 0u64;

            for resource_log in &req.resource_logs {
                for scope_log in &resource_log.scope_logs {
                    for record in &scope_log.log_records {
                        log_count += 1;
                        let body = record.body.string_value.clone().unwrap_or_default();
                        let severity = match OTelSeverityNumber::try_from(record.severity_number) {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Invalid severity number: {}", e);
                                continue;
                            }
                        };
                        let time_unix_nano = match record.time_unix_nano.parse() {
                            Ok(t) => t,
                            Err(e) => {
                                error!("Invalid time_unix_nano '{}': {}", record.time_unix_nano, e);
                                continue;
                            }
                        };

                        let log = OTelLogRecord::new(
                            time_unix_nano,
                            severity,
                            body.clone(),
                        );

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

            state.metrics.inc_logs_ingested(log_count);
            state.metrics.inc_logs_classified(classified.len() as u64);

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
            (axum::http::StatusCode::BAD_REQUEST, response).into_response()
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct OtlpMetricsRequest {
    #[serde(rename = "resourceMetrics")]
    resource_metrics: Vec<OtlpResourceMetrics>,
}

#[derive(Debug, serde::Deserialize)]
struct OtlpResourceMetrics {
    #[serde(rename = "scopeMetrics", default)]
    scope_metrics: Vec<OtlpScopeMetrics>,
}

#[derive(Debug, serde::Deserialize)]
struct OtlpScopeMetrics {
    #[serde(default)]
    metrics: Vec<OtlpMetric>,
}

#[derive(Debug, serde::Deserialize)]
struct OtlpMetric {
    name: String,
}

#[instrument]
async fn otlp_metrics_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let request = match serde_json::from_value::<OtlpMetricsRequest>(payload) {
        Ok(request) if !request.resource_metrics.is_empty() => request,
        Ok(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "accepted": false,
                    "signal": "metric",
                    "error": "invalid OTLP metric payload: resourceMetrics must be a non-empty array"
                })),
            )
                .into_response();
        }
        Err(error) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "accepted": false,
                    "signal": "metric",
                    "error": format!("invalid OTLP metric payload: {error}")
                })),
            )
                .into_response();
        }
    };

    let resource_count = request.resource_metrics.len();
    let metric_count = request
        .resource_metrics
        .iter()
        .flat_map(|resource| resource.scope_metrics.iter())
        .flat_map(|scope| scope.metrics.iter())
        .filter(|metric| !metric.name.is_empty())
        .count();

    state.metrics.inc_metrics_ingested(metric_count as u64);

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
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let resource_count = payload
        .get("resourceSpans")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    state.metrics.inc_traces_ingested(resource_count as u64);

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

#[derive(Serialize, Deserialize, Debug)]
struct EvaluateSloRequest {
    gate_name: String,
    expression: String,
    log_shape_observed: bool,
    metric_observed: bool,
    slo_expected: bool,
}

#[instrument]
async fn evaluate_slo_handler(
    Json(req): Json<EvaluateSloRequest>,
) -> impl IntoResponse {
    let mut registry = MetricRegistry::new();
    let report = check_otel_logic_slo_as_sarif(
        &mut registry,
        &req.gate_name,
        &req.expression,
        req.log_shape_observed,
        req.metric_observed,
        req.slo_expected,
    );

    let status = if report.has_errors() {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::OK
    };

    (status, Json(report)).into_response()
}

pub fn create_app() -> Result<Router, anyhow::Error> {
    let state = Arc::new(AppState::new()?);

    Ok(Router::new()
        .route("/", get(dashboard_handler))
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .route("/ws/telemetry", get(websocket_handler))
        .route("/v1/logs", post(otlp_logs_handler))
        .route("/v1/metrics", post(otlp_metrics_handler))
        .route("/v1/traces", post(otlp_traces_handler))
        .route("/rotel/evaluate", post(evaluate_slo_handler))
        .with_state(state))
}

pub async fn run_server() -> Result<(), anyhow::Error> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to 0.0.0.0:8080: {e}"))?;
    info!("Rotel Visual OTel Surface starting on 0.0.0.0:8080");

    axum::serve(listener, create_app()?)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {e}"))?;
    Ok(())
}
