use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::settings::ChatSettings;

pub const INTERNAL_OPENAI_ADDR: &str = "127.0.0.1:15115";
pub const INTERNAL_OPENAI_CHAT_URL: &str = "http://127.0.0.1:15115/v1/chat/completions";
pub const INTERNAL_DOCS_URL: &str = "http://127.0.0.1:15115/docs/";
pub const INTERNAL_PHI_MODEL: &str = "phi-4-mini-reasoning";
pub const INTERNAL_LOCAL_API_KEY: &str = "local-tool-tray";
pub const DEFAULT_CLOUD_CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Error)]
pub enum InternalOpenAiError {
    #[error("failed to bind internal OpenAI endpoint at {addr}: {source}")]
    Bind {
        addr: String,
        source: std::io::Error,
    },
    #[error("internal endpoint thread failed: {0}")]
    Thread(String),
}

#[derive(Debug)]
pub struct InternalOpenAiHandle {
    addr: String,
    shutdown_tx: mpsc::Sender<()>,
    join: Option<thread::JoinHandle<()>>,
}

impl InternalOpenAiHandle {
    pub fn chat_url(&self) -> String {
        format!("http://{}/v1/chat/completions", self.addr)
    }

    pub fn docs_url(&self) -> String {
        format!("http://{}/docs/", self.addr)
    }
}

impl Drop for InternalOpenAiHandle {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

pub trait InternalChatBackend: std::fmt::Debug + Send + Sync + 'static {
    fn complete(&self, request: &OpenAiChatRequest) -> Result<String, String>;
}

pub fn internal_phi_chat_settings(system_prompt: impl Into<String>) -> ChatSettings {
    ChatSettings {
        endpoint_url: INTERNAL_OPENAI_CHAT_URL.to_string(),
        api_key: INTERNAL_LOCAL_API_KEY.to_string(),
        model: INTERNAL_PHI_MODEL.to_string(),
        system_prompt: system_prompt.into(),
    }
}

pub fn cloud_chat_settings(system_prompt: impl Into<String>) -> ChatSettings {
    ChatSettings {
        endpoint_url: DEFAULT_CLOUD_CHAT_URL.to_string(),
        api_key: String::new(),
        model: String::new(),
        system_prompt: system_prompt.into(),
    }
}

pub fn internal_phi_backend_status() -> String {
    let mut lines = vec![
        format!("model: {INTERNAL_PHI_MODEL}"),
        format!("openai_endpoint: {INTERNAL_OPENAI_CHAT_URL}"),
        "rig_client: RigAgentRuntime".to_string(),
    ];

    #[cfg(feature = "mistralrs-llm")]
    {
        let model_path = default_phi4_model_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "not found".to_string());
        lines.push(format!("mistralrs: compiled, phi4_gguf: {model_path}"));
    }

    #[cfg(not(feature = "mistralrs-llm"))]
    lines.push("mistralrs: not compiled in this build".to_string());

    #[cfg(feature = "local-llm")]
    lines.push("candle: compiled in this build".to_string());

    #[cfg(not(feature = "local-llm"))]
    lines.push("candle: not compiled in this build".to_string());

    lines.push(
        "fallback: deterministic Phi-4-compatible local endpoint when model runtime is unavailable"
            .to_string(),
    );
    lines.join("\n")
}

pub fn docs_playbook_status() -> String {
    match default_docs_root() {
        Some(root) if root.join("index.html").exists() => {
            format!("Docs playbook ready at {INTERNAL_DOCS_URL}\nroot: {}", root.display())
        }
        Some(root) => format!(
            "Docs playbook root exists but index.html is missing at {}. Run `just docgen` to rebuild the mdBook output.",
            root.display()
        ),
        None => "Docs playbook is not built. Run `just docgen` to generate book/book before opening the local docs route.".to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct InternalServerConfig {
    pub addr: String,
    pub docs_root: Option<PathBuf>,
}

impl Default for InternalServerConfig {
    fn default() -> Self {
        Self {
            addr: INTERNAL_OPENAI_ADDR.to_string(),
            docs_root: default_docs_root(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Phi4LocalFallbackBackend;

impl InternalChatBackend for Phi4LocalFallbackBackend {
    fn complete(&self, request: &OpenAiChatRequest) -> Result<String, String> {
        let user = request
            .messages
            .iter()
            .rev()
            .find(|message| message.role == "user")
            .map(|message| message.content_text())
            .unwrap_or_default();

        let response = if user.contains("audit_playbook")
            || user.contains("audit playbook")
            || user.contains("visual evidence graph")
        {
            serde_json::json!({
                "playbook": "audit_playbook",
                "mode": "deterministic_fallback",
                "steps": [
                    "ingest_rows",
                    "classify_transactions",
                    "phi4_edge_proposals",
                    "operator_review",
                    "workbook_export",
                    "evidence_chain",
                    "visual_audit_graph"
                ],
                "requires_model_assets": false
            })
            .to_string()
        } else if user.contains("\"job\":\"classify_transaction\"")
            || user.contains("classify_transaction") && user.contains("return_schema")
        {
            serde_json::json!({
                "category": "Meals",
                "confidence": 0.72,
                "reason": "deterministic Phi-4 fallback classification",
                "suggested_tags": ["#phi4-fallback"]
            })
            .to_string()
        } else if user.contains("fn ") || user.contains("if ") || user.contains("match ") {
            [
                "fn classify_rows() -> score_confidence",
                "if confidence > 0.85 -> commit_workbook",
                "if confidence > 0.60 -> review_flag",
                "if confidence <= 0.60 -> escalate_operator",
                "fn review_flag() -> commit_workbook",
                "",
                "The internal phi-4-mini-reasoning endpoint preserved the supported Rhai DSL and added a review-safe medium-confidence lane.",
            ]
            .join("\n")
        } else {
            format!(
                "Internal phi-4-mini-reasoning endpoint is online. Received {} message(s). Build with the local model feature and configure the GGUF path to replace this deterministic fallback with real Phi-4 inference.",
                request.messages.len()
            )
        };

        Ok(response)
    }
}

#[derive(Debug, Deserialize)]
pub struct OpenAiChatRequest {
    pub model: String,
    #[serde(default)]
    pub messages: Vec<OpenAiChatMessage>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct OpenAiChatMessage {
    pub role: String,
    pub content: OpenAiMessageContent,
}

impl OpenAiChatMessage {
    fn content_text(&self) -> String {
        self.content.text()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum OpenAiMessageContent {
    Text(String),
    Parts(Vec<serde_json::Value>),
}

impl OpenAiMessageContent {
    fn text(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Parts(parts) => parts
                .iter()
                .filter_map(|part| match part {
                    serde_json::Value::String(text) => Some(text.as_str()),
                    serde_json::Value::Object(object) => {
                        object.get("text").and_then(serde_json::Value::as_str)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

impl From<String> for OpenAiMessageContent {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for OpenAiMessageContent {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

#[derive(Debug, Serialize)]
struct OpenAiChatResponse {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

#[derive(Debug, Serialize)]
struct OpenAiChoice {
    index: usize,
    message: OpenAiChatMessage,
    finish_reason: &'static str,
}

#[derive(Debug, Serialize)]
struct OpenAiUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[derive(Debug, Serialize)]
struct OpenAiModelList {
    object: &'static str,
    data: Vec<OpenAiModel>,
}

#[derive(Debug, Serialize)]
struct OpenAiModel {
    id: &'static str,
    object: &'static str,
    owned_by: &'static str,
}

pub fn spawn_internal_openai_endpoint(
    addr: impl Into<String>,
    backend: Arc<dyn InternalChatBackend>,
) -> Result<InternalOpenAiHandle, InternalOpenAiError> {
    spawn_internal_openai_endpoint_with_config(
        InternalServerConfig {
            addr: addr.into(),
            docs_root: default_docs_root(),
        },
        backend,
    )
}

pub fn spawn_internal_openai_endpoint_with_config(
    config: InternalServerConfig,
    backend: Arc<dyn InternalChatBackend>,
) -> Result<InternalOpenAiHandle, InternalOpenAiError> {
    let addr = config.addr;
    let listener = TcpListener::bind(&addr).map_err(|source| InternalOpenAiError::Bind {
        addr: addr.clone(),
        source,
    })?;
    listener
        .set_nonblocking(true)
        .map_err(|source| InternalOpenAiError::Bind {
            addr: addr.clone(),
            source,
        })?;

    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    let thread_addr = addr.clone();
    let join = thread::Builder::new()
        .name("ledgerr-internal-openai".to_string())
        .spawn(move || serve_loop(listener, shutdown_rx, backend, config.docs_root))
        .map_err(|error| InternalOpenAiError::Thread(error.to_string()))?;

    Ok(InternalOpenAiHandle {
        addr: thread_addr,
        shutdown_tx,
        join: Some(join),
    })
}

pub fn start_default_internal_openai_endpoint() -> Result<InternalOpenAiHandle, InternalOpenAiError>
{
    spawn_internal_openai_endpoint_with_config(
        InternalServerConfig::default(),
        default_internal_backend(),
    )
}

fn default_internal_backend() -> Arc<dyn InternalChatBackend> {
    #[cfg(feature = "mistralrs-llm")]
    {
        if let Some(path) = default_phi4_model_path() {
            if let Ok(runtime) = crate::local_llm_mistral::LocalMistralRuntime::new(path) {
                return Arc::new(Phi4MistralBackend { runtime });
            }
        }
    }

    Arc::new(Phi4LocalFallbackBackend::default())
}

#[cfg(feature = "mistralrs-llm")]
#[derive(Debug)]
struct Phi4MistralBackend {
    runtime: crate::local_llm_mistral::LocalMistralRuntime,
}

#[cfg(feature = "mistralrs-llm")]
impl InternalChatBackend for Phi4MistralBackend {
    fn complete(&self, request: &OpenAiChatRequest) -> Result<String, String> {
        use crate::agent_runtime::{AgentRuntime, ModelRequest, ModelRole, ModelTurn};

        let mut system_prompt = None;
        let mut history = Vec::new();
        let mut user_message = None;

        for message in &request.messages {
            match message.role.as_str() {
                "system" if system_prompt.is_none() => {
                    system_prompt = Some(message.content_text());
                }
                "assistant" => history.push(ModelTurn {
                    role: ModelRole::Assistant,
                    content: message.content_text(),
                }),
                "user" => {
                    if let Some(previous_user) = user_message.replace(message.content_text()) {
                        history.push(ModelTurn {
                            role: ModelRole::User,
                            content: previous_user,
                        });
                    }
                }
                _ => {}
            }
        }

        let mut model_request =
            ModelRequest::text(user_message.unwrap_or_else(|| "Continue.".to_string()))
                .with_history(history);
        if let Some(system_prompt) = system_prompt {
            model_request = model_request.with_system_prompt(system_prompt);
        }
        if let Some(max_tokens) = request.max_tokens {
            model_request = model_request.with_max_tokens(max_tokens);
        }

        AgentRuntime::complete(&self.runtime, model_request)
            .map(|response| response.assistant_text)
            .map_err(|error| error.to_string())
    }
}

fn serve_loop(
    listener: TcpListener,
    shutdown_rx: mpsc::Receiver<()>,
    backend: Arc<dyn InternalChatBackend>,
    docs_root: Option<PathBuf>,
) {
    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }
        match listener.accept() {
            Ok((stream, _)) => handle_stream(stream, backend.as_ref(), docs_root.as_deref()),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(25));
            }
            Err(_) => break,
        }
    }
}

fn handle_stream(
    mut stream: TcpStream,
    backend: &dyn InternalChatBackend,
    docs_root: Option<&Path>,
) {
    let mut buffer = Vec::with_capacity(8192);
    let mut chunk = [0_u8; 2048];
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                if request_complete(&buffer) {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let response = route_http_request(&buffer, backend, docs_root);
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn request_complete(buffer: &[u8]) -> bool {
    let Some(header_end) = find_header_end(buffer) else {
        return false;
    };
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let content_length = parse_content_length(&headers).unwrap_or_default();
    buffer.len() >= header_end + 4 + content_length
}

fn route_http_request(
    raw: &[u8],
    backend: &dyn InternalChatBackend,
    docs_root: Option<&Path>,
) -> String {
    let Some(header_end) = find_header_end(raw) else {
        return json_response(400, &serde_json::json!({ "error": "invalid request" }));
    };
    let headers = String::from_utf8_lossy(&raw[..header_end]);
    let request_line = headers.lines().next().unwrap_or_default();
    let body = &raw[header_end + 4..];

    if request_line.starts_with("GET /docs ") {
        return redirect_response("/docs/");
    }

    if request_line.starts_with("GET /docs/ ") || request_line.starts_with("GET /docs/") {
        return match docs_root {
            Some(root) => serve_docs_request(request_line, root),
            None => docs_missing_response(),
        };
    }

    if request_line.starts_with("GET /v1/models ") {
        let payload = OpenAiModelList {
            object: "list",
            data: vec![OpenAiModel {
                id: INTERNAL_PHI_MODEL,
                object: "model",
                owned_by: "l3dg3rr",
            }],
        };
        return json_response(200, &payload);
    }

    if !request_line.starts_with("POST /v1/chat/completions ") {
        return json_response(404, &serde_json::json!({ "error": "not found" }));
    }

    let request = match serde_json::from_slice::<OpenAiChatRequest>(body) {
        Ok(request) => request,
        Err(error) => {
            return json_response(
                400,
                &serde_json::json!({ "error": format!("invalid chat request: {error}") }),
            );
        }
    };

    if request.stream {
        return json_response(
            400,
            &serde_json::json!({ "error": "streaming responses are not supported by the internal endpoint yet" }),
        );
    }

    let assistant_text = match backend.complete(&request) {
        Ok(text) => text,
        Err(error) => return json_response(500, &serde_json::json!({ "error": error })),
    };
    json_response(200, &chat_response(&request, assistant_text))
}

pub fn open_internal_docs_in_browser() -> std::io::Result<()> {
    open_url_in_browser(INTERNAL_DOCS_URL)
}

pub fn open_url_in_browser(url: &str) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map(|_| ())
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn().map(|_| ())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(url).spawn().map(|_| ())
    }
}

fn serve_docs_request(request_line: &str, docs_root: &Path) -> String {
    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/docs/")
        .trim_start_matches("/docs/");
    let relative = if path.is_empty() { "index.html" } else { path };
    let Some(safe_path) = safe_join_docs_path(docs_root, relative) else {
        return json_response(400, &serde_json::json!({ "error": "invalid docs path" }));
    };
    let file_path = if safe_path.is_dir() {
        safe_path.join("index.html")
    } else {
        safe_path
    };
    match std::fs::read(&file_path) {
        Ok(bytes) => bytes_response(200, mime_type(&file_path), &bytes),
        Err(_) if relative == "index.html" => docs_missing_response(),
        Err(_) => json_response(404, &serde_json::json!({ "error": "docs asset not found" })),
    }
}

fn docs_missing_response() -> String {
    bytes_response(
        404,
        "text/html; charset=utf-8",
        br#"<!doctype html>
<html>
<head><meta charset="utf-8"><title>l3dg3rr docs playbook</title></head>
<body style="font-family: system-ui, sans-serif; margin: 2rem; line-height: 1.5;">
<h1>Docs playbook is not built</h1>
<p>The internal docs route is active, but <code>book/book/index.html</code> was not found.</p>
<p>Run <code>just docgen</code>, then reload <code>http://127.0.0.1:15115/docs/</code>.</p>
</body>
</html>"#,
    )
}

fn safe_join_docs_path(root: &Path, relative: &str) -> Option<PathBuf> {
    let mut out = root.to_path_buf();
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}

fn mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "svg" => "image/svg+xml",
        "json" => "application/json",
        "wasm" => "application/wasm",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}

fn redirect_response(location: &str) -> String {
    format!(
        "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
}

fn bytes_response(status: u16, content_type: &str, body: &[u8]) -> String {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "OK",
    };
    let mut response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    String::from_utf8_lossy(&response).into_owned()
}

fn chat_response(request: &OpenAiChatRequest, assistant_text: String) -> OpenAiChatResponse {
    let prompt_tokens = request
        .messages
        .iter()
        .map(|message| estimate_tokens(&message.content_text()))
        .sum::<usize>();
    let completion_tokens = estimate_tokens(&assistant_text);

    OpenAiChatResponse {
        id: format!("chatcmpl-l3dg3rr-{}", unix_timestamp()),
        object: "chat.completion",
        created: unix_timestamp(),
        model: if request.model.trim().is_empty() {
            INTERNAL_PHI_MODEL.to_string()
        } else {
            request.model.trim().to_string()
        },
        choices: vec![OpenAiChoice {
            index: 0,
            message: OpenAiChatMessage {
                role: "assistant".to_string(),
                content: assistant_text.into(),
            },
            finish_reason: "stop",
        }],
        usage: OpenAiUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        },
    }
}

fn json_response(status: u16, payload: &impl Serialize) -> String {
    let body = serde_json::to_string(payload)
        .unwrap_or_else(|_| "{\"error\":\"serialization failure\"}".to_string());
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(headers: &str) -> Option<usize> {
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse().ok())
            .flatten()
    })
}

fn estimate_tokens(text: &str) -> usize {
    text.split_whitespace().count().max(1)
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn default_docs_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .map(|workspace| workspace.join("book/book"))?;
    root.exists().then_some(root)
}

#[cfg(feature = "mistralrs-llm")]
fn default_phi4_model_path() -> Option<PathBuf> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)?;
    let repo_model =
        workspace.join("models/unsloth/Phi-4-mini-reasoning-GGUF/Phi-4-mini-reasoning-Q3_K_M.gguf");
    if repo_model.exists() {
        return Some(repo_model);
    }

    let d_drive_model = PathBuf::from(
        "/mnt/d/models/unsloth/Phi-4-mini-reasoning-GGUF/Phi-4-mini-reasoning-Q3_K_M.gguf",
    );
    d_drive_model.exists().then_some(d_drive_model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_runtime::{
        ClassifyTransactionJob, TransactionClassificationOutput, PHI4_TYPED_JOB_SYSTEM_PROMPT,
    };

    #[derive(Debug)]
    struct FixedBackend;

    impl InternalChatBackend for FixedBackend {
        fn complete(&self, request: &OpenAiChatRequest) -> Result<String, String> {
            Ok(format!(
                "model={} messages={}",
                request.model,
                request.messages.len()
            ))
        }
    }

    #[test]
    fn chat_completion_returns_openai_compatible_response() {
        let body = serde_json::json!({
            "model": "phi-4-mini-reasoning",
            "messages": [
                { "role": "system", "content": "be brief" },
                { "role": "user", "content": "hello" }
            ],
            "max_tokens": 64
        })
        .to_string();
        let raw = format!(
            "POST /v1/chat/completions HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );

        let response = route_http_request(raw.as_bytes(), &FixedBackend, None);

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"object\":\"chat.completion\""));
        assert!(response.contains("\"model\":\"phi-4-mini-reasoning\""));
        assert!(response.contains("model=phi-4-mini-reasoning messages=2"));
    }

    #[test]
    fn chat_completion_accepts_openai_content_part_arrays() {
        let body = serde_json::json!({
            "model": "phi-4-mini-reasoning",
            "messages": [
                {
                    "role": "user",
                    "content": [
                        { "type": "text", "text": "fn classify_rows() -> score_confidence" }
                    ]
                }
            ]
        })
        .to_string();
        let raw = format!(
            "POST /v1/chat/completions HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );

        let response = route_http_request(raw.as_bytes(), &Phi4LocalFallbackBackend, None);

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(
            response.contains("if confidence &gt; 0.60 -> review_flag")
                || response.contains("if confidence > 0.60 -> review_flag")
        );
    }

    #[test]
    fn models_route_lists_internal_phi_model() {
        let raw = "GET /v1/models HTTP/1.1\r\nHost: localhost\r\n\r\n";

        let response = route_http_request(raw.as_bytes(), &FixedBackend, None);

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"id\":\"phi-4-mini-reasoning\""));
    }

    #[test]
    fn fallback_backend_generates_review_safe_rhai_when_prompt_contains_rules() {
        let request = OpenAiChatRequest {
            model: INTERNAL_PHI_MODEL.to_string(),
            messages: vec![OpenAiChatMessage {
                role: "user".to_string(),
                content: "fn classify_rows() -> score_confidence".into(),
            }],
            max_tokens: Some(128),
            stream: false,
        };

        let response = Phi4LocalFallbackBackend::default()
            .complete(&request)
            .expect("fallback should respond");

        assert!(response.contains("if confidence > 0.60 -> review_flag"));
        assert!(response.contains("review-safe"));
    }

    #[test]
    fn fallback_backend_generates_valid_typed_classification_json() {
        let job = ClassifyTransactionJob {
            tx_id: "tx_123".to_string(),
            account_id: "WF-BH-CHK".to_string(),
            date: "2024-01-31".to_string(),
            amount: "-12.34".to_string(),
            description: "Cafe lunch".to_string(),
        };
        let model_request = job.to_model_request().expect("model request");
        let request = OpenAiChatRequest {
            model: INTERNAL_PHI_MODEL.to_string(),
            messages: vec![
                OpenAiChatMessage {
                    role: "system".to_string(),
                    content: PHI4_TYPED_JOB_SYSTEM_PROMPT.into(),
                },
                OpenAiChatMessage {
                    role: "user".to_string(),
                    content: model_request.user_message.into(),
                },
            ],
            max_tokens: model_request.max_tokens,
            stream: false,
        };

        let response = Phi4LocalFallbackBackend::default()
            .complete(&request)
            .expect("fallback should respond");
        let output: TransactionClassificationOutput =
            serde_json::from_str(&response).expect("typed json");

        output.validate().expect("valid typed output");
        assert_eq!(output.category, "Meals");
        assert_eq!(output.suggested_tags, ["#phi4-fallback"]);
    }

    #[test]
    fn internal_phi_fallback_runs_audit_playbook_prompt() {
        let request = OpenAiChatRequest {
            model: INTERNAL_PHI_MODEL.to_string(),
            messages: vec![OpenAiChatMessage {
                role: "user".to_string(),
                content: "Run the audit playbook and return the visual evidence graph steps."
                    .into(),
            }],
            max_tokens: Some(256),
            stream: false,
        };

        let response = Phi4LocalFallbackBackend::default()
            .complete(&request)
            .expect("fallback should respond");
        let payload: serde_json::Value = serde_json::from_str(&response).expect("json response");

        assert_eq!(payload["playbook"], "audit_playbook");
        assert_eq!(payload["mode"], "deterministic_fallback");
        assert_eq!(payload["requires_model_assets"], false);
        assert!(payload["steps"]
            .as_array()
            .expect("steps")
            .iter()
            .any(|step| step == "visual_audit_graph"));
    }

    #[test]
    fn provider_switch_settings_point_to_internal_or_cloud_endpoint() {
        let internal = internal_phi_chat_settings("system");
        assert_eq!(internal.endpoint_url, INTERNAL_OPENAI_CHAT_URL);
        assert_eq!(internal.model, INTERNAL_PHI_MODEL);
        assert_eq!(internal.api_key, INTERNAL_LOCAL_API_KEY);

        let cloud = cloud_chat_settings("system");
        assert_eq!(cloud.endpoint_url, DEFAULT_CLOUD_CHAT_URL);
        assert!(cloud.model.is_empty());
        assert!(cloud.api_key.is_empty());
    }

    #[test]
    fn backend_status_names_rig_phi4_mistralrs_and_candle() {
        let status = internal_phi_backend_status();

        assert!(status.contains("model: phi-4-mini-reasoning"));
        assert!(status.contains("openai_endpoint: http://127.0.0.1:15115/v1/chat/completions"));
        assert!(status.contains("rig_client: RigAgentRuntime"));
        assert!(status.contains("mistralrs:"));
        assert!(status.contains("candle:"));
    }

    #[test]
    fn docs_route_serves_index_from_configured_root() {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::write(temp.path().join("index.html"), "<h1>Playbook</h1>").expect("write docs");
        let raw = "GET /docs/ HTTP/1.1\r\nHost: localhost\r\n\r\n";

        let response = route_http_request(raw.as_bytes(), &FixedBackend, Some(temp.path()));

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("text/html"));
        assert!(response.contains("<h1>Playbook</h1>"));
    }

    #[test]
    fn docs_route_renders_html_diagnostic_when_book_is_missing() {
        let raw = "GET /docs/ HTTP/1.1\r\nHost: localhost\r\n\r\n";

        let response = route_http_request(raw.as_bytes(), &FixedBackend, None);

        assert!(response.starts_with("HTTP/1.1 404 Not Found"));
        assert!(response.contains("text/html"));
        assert!(response.contains("Docs playbook is not built"));
    }

    #[test]
    fn docs_route_rejects_parent_traversal() {
        let raw = "GET /docs/../settings.json HTTP/1.1\r\nHost: localhost\r\n\r\n";

        let response = route_http_request(raw.as_bytes(), &FixedBackend, Some(Path::new(".")));

        assert!(response.starts_with("HTTP/1.1 400 Bad Request"));
    }
}
