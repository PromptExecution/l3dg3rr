use thiserror::Error;

use crate::agent_runtime::{
    AgentRuntime, AgentRuntimeError, ModelRequest, ModelRole, ModelTurn, RigAgentRuntime,
};
use crate::settings::ChatSettings;

pub const RHAI_RULE_SYSTEM_PROMPT: &str = "You are the l3dg3rr Rhai rule editor. Return only supported documentation DSL lines unless explicitly asked for explanation. Supported lines are `fn source() -> target`, `if expression -> target`, and `match expr => Arm -> target`. Preserve financial audit safety: do not bypass confidence, review, or commit approval gates.";
pub const DEFAULT_RHAI_RULE_MODEL: &str = "phi-4-mini-reasoning";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatTurn {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionDiff {
    pub field: String,
    pub before: String,
    pub after: String,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewLogEntry {
    pub action: String,
    pub summary: String,
    pub diffs: Vec<DecisionDiff>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReviewLog {
    entries: Vec<ReviewLogEntry>,
}

impl ReviewLog {
    pub fn push(&mut self, entry: ReviewLogEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[ReviewLogEntry] {
        &self.entries
    }

    pub fn render(&self) -> String {
        render_review_log(&self.entries)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RigPromptPreview {
    pub endpoint_url: String,
    pub model: String,
    pub messages_json: String,
}

impl RigPromptPreview {
    pub fn render(&self) -> String {
        format!(
            "Rig OpenAI-compatible request\nPOST {}\nmodel: {}\n\n{}",
            self.endpoint_url, self.model, self.messages_json
        )
    }
}

pub fn render_rig_exchange_log(
    preview: &RigPromptPreview,
    backend_status: &str,
    response: Option<&str>,
    error: Option<&str>,
) -> String {
    let mut lines = vec![
        preview.render(),
        String::new(),
        "Rig/OpenAI backend status".to_string(),
        backend_status.trim().to_string(),
        String::new(),
        "Rig/OpenAI response".to_string(),
    ];

    match (response, error) {
        (Some(response), _) => lines.push(response.trim().to_string()),
        (_, Some(error)) => lines.push(format!("ERROR: {}", error.trim())),
        (None, None) => lines.push("Awaiting response...".to_string()),
    }

    lines.join("\n")
}

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("chat endpoint is empty")]
    MissingEndpoint,
    #[error("chat model is empty")]
    MissingModel,
    #[error("api key is empty")]
    MissingApiKey,
    #[error("message is empty")]
    EmptyMessage,
    #[error("failed to create async runtime: {0}")]
    Runtime(std::io::Error),
    #[error("chat request failed: {0}")]
    Rig(rig::completion::CompletionError),
    #[error("chat client setup failed: {0}")]
    RigHttp(rig::http_client::Error),
    #[error("response did not contain an assistant message")]
    MissingAssistantMessage,
    #[error("failed to parse structured model response: {0}")]
    Parse(serde_json::Error),
    #[error("local llm error: {0}")]
    LocalLlm(String),
}

pub fn send_chat_message(
    settings: &ChatSettings,
    history: &[ChatTurn],
    pending_message: &str,
) -> Result<String, ChatError> {
    let request = ModelRequest::text(pending_message)
        .with_system_prompt(settings.system_prompt.clone())
        .with_history(history.iter().map(model_turn));
    let runtime = RigAgentRuntime::new(settings.clone());
    let response = runtime.complete(request).map_err(ChatError::from)?;
    Ok(response.assistant_text)
}

pub fn build_rig_prompt_preview(
    settings: &ChatSettings,
    history: &[ChatTurn],
    pending_message: &str,
) -> RigPromptPreview {
    let mut messages = Vec::new();
    let system_prompt = settings.system_prompt.trim();
    if !system_prompt.is_empty() {
        messages.push(serde_json::json!({
            "role": "system",
            "content": system_prompt,
        }));
    }
    for turn in history {
        let content = turn.content.trim();
        if content.is_empty() {
            continue;
        }
        messages.push(serde_json::json!({
            "role": chat_role_name(turn.role),
            "content": content,
        }));
    }
    messages.push(serde_json::json!({
        "role": "user",
        "content": pending_message.trim(),
    }));

    let payload = serde_json::json!({
        "model": settings.model.trim(),
        "messages": messages,
        "stream": false,
    });

    RigPromptPreview {
        endpoint_url: settings.endpoint_url.trim().to_string(),
        model: settings.model.trim().to_string(),
        messages_json: serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string()),
    }
}

pub fn render_transcript(history: &[ChatTurn]) -> String {
    if history.is_empty() {
        return "No messages yet.".to_string();
    }

    history
        .iter()
        .map(|turn| {
            let speaker = match turn.role {
                ChatRole::System => "System",
                ChatRole::User => "You",
                ChatRole::Assistant => "Assistant",
            };
            format!("{speaker}\n{}\n", turn.content.trim())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn chat_role_name(role: ChatRole) -> &'static str {
    match role {
        ChatRole::System => "system",
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
    }
}

pub fn rhai_rule_prompt_seed() -> &'static str {
    "Mutate this Rhai workflow to add a review path for medium-confidence classifications and explain the change in one short paragraph:\n\nfn ingest_pdf() -> detect_shape\nfn detect_shape() -> classify_rows\nif confidence > 0.85 -> commit_workbook\nif confidence <= 0.85 -> review_flag\nfn review_flag() -> commit_workbook"
}

pub fn rhai_rule_prompt_seed_log(
    previous_model: &str,
    previous_system_prompt: &str,
) -> ReviewLogEntry {
    let mut diffs = Vec::new();
    if previous_model.trim().is_empty() {
        diffs.push(DecisionDiff {
            field: "chat.model".to_string(),
            before: "<empty>".to_string(),
            after: DEFAULT_RHAI_RULE_MODEL.to_string(),
            rationale: "Use the default local Phi-family example target for rule mutation prompts."
                .to_string(),
        });
    }
    if previous_system_prompt.trim() != RHAI_RULE_SYSTEM_PROMPT {
        diffs.push(DecisionDiff {
            field: "chat.system_prompt".to_string(),
            before: summarize_value(previous_system_prompt),
            after: summarize_value(RHAI_RULE_SYSTEM_PROMPT),
            rationale: "Constrain model output to supported Rhai DSL and audit-safe review gates."
                .to_string(),
        });
    }

    ReviewLogEntry {
        action: "seed_rhai_rule_prompt".to_string(),
        summary: "Prepared the chat surface for Rhai rule mutation review.".to_string(),
        diffs,
    }
}

pub fn user_request_log(message: &str) -> ReviewLogEntry {
    ReviewLogEntry {
        action: "submit_chat_request".to_string(),
        summary: summarize_value(message),
        diffs: vec![DecisionDiff {
            field: "pending_request".to_string(),
            before: "<none>".to_string(),
            after: summarize_value(message),
            rationale: "Capture the operator request that produced the next model response."
                .to_string(),
        }],
    }
}

pub fn assistant_decision_log(previous_rhai: &str, assistant_text: &str) -> ReviewLogEntry {
    let proposed = extract_rhai_decision_lines(assistant_text);
    let before = extract_rhai_decision_lines(previous_rhai);
    let diffs = diff_decision_lines(&before, &proposed);
    let summary = if proposed.is_empty() {
        "Assistant response did not contain supported Rhai DSL decision lines.".to_string()
    } else {
        format!(
            "Assistant proposed {} supported Rhai decision line(s).",
            proposed.len()
        )
    };

    ReviewLogEntry {
        action: "assistant_decision_diff".to_string(),
        summary,
        diffs,
    }
}

pub fn render_review_log(entries: &[ReviewLogEntry]) -> String {
    if entries.is_empty() {
        return "No review log entries yet.".to_string();
    }

    entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let mut out = format!(
                "#{idx}: {}\n{}\n",
                entry.action,
                entry.summary,
                idx = idx + 1
            );
            if entry.diffs.is_empty() {
                out.push_str("Diffset: no field changes detected.\n");
            } else {
                out.push_str("Diffset:\n");
                for diff in &entry.diffs {
                    out.push_str(&format!(
                        "- {}: {} -> {}\n  because {}\n",
                        diff.field, diff.before, diff.after, diff.rationale
                    ));
                }
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn extract_rhai_decision_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("fn ") || line.starts_with("if ") || line.starts_with("match ")
        })
        .map(str::to_string)
        .collect()
}

fn diff_decision_lines(before: &[String], after: &[String]) -> Vec<DecisionDiff> {
    let mut diffs = Vec::new();
    for line in after {
        if !before.contains(line) {
            diffs.push(DecisionDiff {
                field: "rhai.decision.added".to_string(),
                before: "<absent>".to_string(),
                after: line.clone(),
                rationale: "Model output introduced a new supported Rhai decision line."
                    .to_string(),
            });
        }
    }
    for line in before {
        if !after.contains(line) {
            diffs.push(DecisionDiff {
                field: "rhai.decision.removed".to_string(),
                before: line.clone(),
                after: "<absent>".to_string(),
                rationale: "Model output omitted a previously visible Rhai decision line."
                    .to_string(),
            });
        }
    }
    diffs
}

fn summarize_value(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "<empty>".to_string();
    }
    const MAX_CHARS: usize = 120;
    let mut summary: String = trimmed.chars().take(MAX_CHARS).collect();
    if trimmed.chars().count() > MAX_CHARS {
        summary.push_str("...");
    }
    summary
}

fn model_turn(turn: &ChatTurn) -> ModelTurn {
    ModelTurn {
        role: match turn.role {
            ChatRole::System => ModelRole::System,
            ChatRole::User => ModelRole::User,
            ChatRole::Assistant => ModelRole::Assistant,
        },
        content: turn.content.clone(),
    }
}

impl From<AgentRuntimeError> for ChatError {
    fn from(value: AgentRuntimeError) -> Self {
        match value {
            AgentRuntimeError::MissingEndpoint => Self::MissingEndpoint,
            AgentRuntimeError::MissingModel => Self::MissingModel,
            AgentRuntimeError::MissingApiKey => Self::MissingApiKey,
            AgentRuntimeError::EmptyMessage => Self::EmptyMessage,
            AgentRuntimeError::Runtime(error) => Self::Runtime(error),
            AgentRuntimeError::Rig(error) => Self::Rig(error),
            AgentRuntimeError::RigHttp(error) => Self::RigHttp(error),
            AgentRuntimeError::MissingAssistantMessage => Self::MissingAssistantMessage,
            AgentRuntimeError::Parse(error) => Self::Parse(error),
            AgentRuntimeError::LocalLlm(msg) => Self::LocalLlm(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_settings() -> ChatSettings {
        ChatSettings {
            endpoint_url: "https://example.test/v1/chat/completions".to_string(),
            api_key: "test-key".to_string(),
            model: "gpt-test".to_string(),
            system_prompt: "You are terse.".to_string(),
        }
    }

    #[test]
    fn empty_fields_are_rejected_before_network() {
        let missing_endpoint = ChatSettings {
            endpoint_url: String::new(),
            ..test_settings()
        };
        assert!(matches!(
            send_chat_message(&missing_endpoint, &[], "hello"),
            Err(ChatError::MissingEndpoint)
        ));

        let missing_model = ChatSettings {
            model: String::new(),
            ..test_settings()
        };
        assert!(matches!(
            send_chat_message(&missing_model, &[], "hello"),
            Err(ChatError::MissingModel)
        ));

        let missing_key = ChatSettings {
            api_key: String::new(),
            ..test_settings()
        };
        assert!(matches!(
            send_chat_message(&missing_key, &[], "hello"),
            Err(ChatError::MissingApiKey)
        ));

        assert!(matches!(
            send_chat_message(&test_settings(), &[], "   "),
            Err(ChatError::EmptyMessage)
        ));
    }

    #[test]
    fn transcript_renders_roles_for_slint_display() {
        let transcript = render_transcript(&[
            ChatTurn {
                role: ChatRole::User,
                content: "mutate rule".to_string(),
            },
            ChatTurn {
                role: ChatRole::Assistant,
                content: "fn classify() -> review".to_string(),
            },
        ]);

        assert!(transcript.contains("You\nmutate rule"));
        assert!(transcript.contains("Assistant\nfn classify() -> review"));
    }

    #[test]
    fn rig_prompt_preview_shows_internal_openai_request_shape() {
        let settings = ChatSettings {
            endpoint_url: "http://127.0.0.1:15115/v1/chat/completions".to_string(),
            api_key: "local-tool-tray".to_string(),
            model: "phi-4-mini-reasoning".to_string(),
            system_prompt: "Use Rhai DSL.".to_string(),
        };
        let preview = build_rig_prompt_preview(
            &settings,
            &[ChatTurn {
                role: ChatRole::Assistant,
                content: "Earlier answer".to_string(),
            }],
            "fn classify_rows() -> score_confidence",
        );

        let rendered = preview.render();
        assert!(rendered.contains("POST http://127.0.0.1:15115/v1/chat/completions"));
        assert!(rendered.contains("phi-4-mini-reasoning"));
        assert!(rendered.contains("\"role\": \"system\""));
        assert!(rendered.contains("\"role\": \"assistant\""));
        assert!(rendered.contains("fn classify_rows() -> score_confidence"));
    }

    #[test]
    fn rig_exchange_log_shows_request_backend_and_response() {
        let preview = RigPromptPreview {
            endpoint_url: "http://127.0.0.1:15115/v1/chat/completions".to_string(),
            model: DEFAULT_RHAI_RULE_MODEL.to_string(),
            messages_json: "{}".to_string(),
        };

        let log = render_rig_exchange_log(
            &preview,
            "mistralrs: not compiled\ncandle: not compiled\nmodel: phi-4-mini-reasoning",
            Some("assistant text"),
            None,
        );

        assert!(log.contains("POST http://127.0.0.1:15115/v1/chat/completions"));
        assert!(log.contains("mistralrs: not compiled"));
        assert!(log.contains("candle: not compiled"));
        assert!(log.contains("assistant text"));
    }

    #[test]
    fn seed_prompt_log_records_model_and_system_prompt_diffset() {
        let entry = rhai_rule_prompt_seed_log("", "old prompt");

        assert_eq!(entry.action, "seed_rhai_rule_prompt");
        assert!(entry
            .diffs
            .iter()
            .any(|diff| { diff.field == "chat.model" && diff.after == DEFAULT_RHAI_RULE_MODEL }));
        assert!(entry
            .diffs
            .iter()
            .any(|diff| diff.field == "chat.system_prompt"));
    }

    #[test]
    fn assistant_decision_log_diffs_supported_rhai_lines() {
        let entry = assistant_decision_log(
            "fn classify_rows() -> score_confidence\nif confidence <= 0.85 -> review_flag",
            "Explanation\n```rhai\nfn classify_rows() -> score_confidence\nif confidence > 0.85 -> commit_workbook\nif confidence > 0.60 -> review_flag\n```",
        );

        assert_eq!(entry.action, "assistant_decision_diff");
        assert!(entry
            .diffs
            .iter()
            .any(|diff| diff.field == "rhai.decision.added"
                && diff.after == "if confidence > 0.60 -> review_flag"));
        assert!(entry
            .diffs
            .iter()
            .any(|diff| diff.field == "rhai.decision.removed"
                && diff.before == "if confidence <= 0.85 -> review_flag"));
    }

    #[test]
    fn review_log_render_is_a_readable_diffset() {
        let mut log = ReviewLog::default();
        log.push(user_request_log("Add a review lane"));

        let rendered = log.render();
        assert!(rendered.contains("#1: submit_chat_request"));
        assert!(rendered.contains("Diffset:"));
        assert!(rendered.contains("pending_request"));
    }
}
