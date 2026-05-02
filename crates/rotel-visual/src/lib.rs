use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
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

#[derive(Debug)]
struct AppState {
    telemetry_tx: broadcast::Sender<TelemetryData>,
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
            .log-entry { margin-bottom: 5px; padding: 5px; border-radius: 3px; }
            .log-entry.error { background-color: #ffe6e6; }
            .log-entry.warn { background-color: #fff3cd; }
            .log-entry.info { background-color: #e7f3ff; }
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
                <h2>System Status</h2>
                <div id="status">Connected</div>
            </div>
        </div>

        <script>
            const logsDiv = document.getElementById('logs');
            const metricsDiv = document.getElementById('metrics');
            const spansDiv = document.getElementById('spans');

            const ws = new WebSocket('ws://' + location.host + '/ws/telemetry');

            ws.onmessage = function(event) {
                const data = JSON.parse(event.data);
                updateLogs(data.logs);
                updateMetrics(data.metrics);
                updateSpans(data.spans);
            };

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
    let mut rx = state.telemetry_tx.subscribe();

    tokio::spawn(async move {
        while let Ok(telemetry) = rx.recv().await {
            let msg = axum::extract::ws::Message::Text(
                serde_json::to_string(&telemetry).unwrap().into()
            );
            if let Err(err) = socket.send(msg).await {
                error!("Error sending telemetry data: {}", err);
                break;
            }
        }
    });
}

pub fn create_app() -> Router {
    let (tx, _rx) = broadcast::channel(100);
    let state = Arc::new(AppState { telemetry_tx: tx });

    Router::new()
        .route("/", get(dashboard_handler))
        .route("/health", get(health_handler))
        .route("/ws/telemetry", get(websocket_handler))
        .with_state(state)
}

pub async fn run_server() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    info!("Rotel Visual OTel Surface starting on 0.0.0.0:8080");

    axum::serve(listener, create_app()).await.unwrap();
}
