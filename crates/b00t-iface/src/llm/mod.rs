//! LLM Machine — internal OpenAI-compatible model serving interface for l3dg3rr.
//!
//! This is the "LLM machine" that b00t surfaces use for inference. It wraps
//! the existing `internal_openai.rs` endpoint with an abstract surface trait,
//! so that the autoresearch loop can autonomously call completions, evaluate
//! prompts, and improve its own code generation quality.
//!
//! # Architecture
//!
//! The LLM machine implements [`ProcessSurface`] so b00t's executive harness
//! governs its lifecycle. It also implements the [`Researcher`] trait
//! (when `autoresearch` feature is active), enabling self-play: the machine
//! calls itself to evaluate prompt variants and keeps the best.

use crate::core::{
    AuditRecord, GovernancePolicy, MaintenanceAction, ProcessSurface, Requirement,
    SurfaceCapability,
};
use crate::AgentRole;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Internal LLM machine configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct LlmMachineConfig {
    /// Base URL for the OpenAI-compatible endpoint.
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    /// Model name to use for completions.
    #[serde(default = "default_model")]
    pub model: String,
    /// Maximum tokens per completion.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Temperature for sampling.
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_endpoint() -> String {
    "http://127.0.0.1:15115/v1/chat/completions".into()
}

fn default_model() -> String {
    "phi-4-mini-reasoning".into()
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f32 {
    0.7
}

impl Default for LlmMachineConfig {
    fn default() -> Self {
        Self {
            endpoint: default_endpoint(),
            model: default_model(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

/// An OpenAI-compatible chat completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// A single message in the chat completion format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// An OpenAI-compatible chat completion response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    #[serde(default)]
    pub usage: ChatUsage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatResponseMessage,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// The LLM machine — manages an internal OpenAI-compatible endpoint.
pub struct LlmMachine {
    pub config: LlmMachineConfig,
    pub total_completions: u64,
    pub total_tokens: u64,
    pub crash_count: u32,
}

#[derive(Debug, Clone)]
pub enum LlmMachineError {
    Endpoint(String),
    Request(String),
    Response(String),
}

impl std::fmt::Display for LlmMachineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Endpoint(e) => write!(f, "endpoint: {e}"),
            Self::Request(e) => write!(f, "request: {e}"),
            Self::Response(e) => write!(f, "response: {e}"),
        }
    }
}

impl std::error::Error for LlmMachineError {}

/// Handle returned by operate() — snapshot of LLM machine stats.
#[derive(Debug, Clone)]
pub struct LlmMachineHandle {
    pub completions: u64,
    pub tokens: u64,
    pub endpoint: String,
    pub model: String,
}

impl LlmMachine {
    pub fn new() -> Self {
        Self {
            config: LlmMachineConfig::default(),
            total_completions: 0,
            total_tokens: 0,
            crash_count: 0,
        }
    }

    pub fn with_config(config: LlmMachineConfig) -> Self {
        Self {
            config,
            total_completions: 0,
            total_tokens: 0,
            crash_count: 0,
        }
    }

    /// Send a chat completion request to the internal endpoint.
    /// This is the core inference operation the autoresearch loop uses.
    pub fn complete(&self, request: ChatRequest) -> Result<ChatResponse, LlmMachineError> {
        let url = &self.config.endpoint;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| LlmMachineError::Endpoint(e.to_string()))?;

        let resp = client
            .post(url)
            .json(&request)
            .send()
            .map_err(|e| LlmMachineError::Request(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(LlmMachineError::Response(format!("HTTP {status}")));
        }

        resp.json::<ChatResponse>()
            .map_err(|e| LlmMachineError::Response(e.to_string()))
    }
}

impl ProcessSurface for LlmMachine {
    type Config = LlmMachineConfig;
    type Error = LlmMachineError;
    type Handle = LlmMachineHandle;

    fn capability(&self) -> SurfaceCapability {
        SurfaceCapability {
            name: "llm-machine",
            requirements: vec![Requirement::PortAvailable(15115)],
            governance: GovernancePolicy {
                allowed_starters: vec![
                    AgentRole::Executive,
                    AgentRole::Operator,
                    AgentRole::Specialist,
                ],
                max_ttl: Duration::from_secs(86400),
                auto_restart: true,
                crash_budget: 10,
            },
        }
    }

    fn init(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.config = config;
        self.total_completions = 0;
        self.total_tokens = 0;
        tracing::info!(
            "LlmMachine initialized: {} @ {}",
            self.config.model,
            self.config.endpoint
        );
        Ok(())
    }

    fn operate(&self) -> Result<Self::Handle, Self::Error> {
        Ok(LlmMachineHandle {
            completions: self.total_completions,
            tokens: self.total_tokens,
            endpoint: self.config.endpoint.clone(),
            model: self.config.model.clone(),
        })
    }

    fn terminate(handle: Self::Handle) -> Result<AuditRecord, Self::Error> {
        Ok(AuditRecord {
            surface_name: "llm-machine".into(),
            uptime: Duration::from_secs(0),
            exit_reason: format!(
                "{} completions, {} tokens",
                handle.completions, handle.tokens
            ),
            crash_count: 0,
            bytes_logged: handle.tokens * 4,
        })
    }

    fn maintain(&self) -> MaintenanceAction {
        if self.crash_count >= self.capability().governance.crash_budget {
            return MaintenanceAction::Quarantine {
                reason: format!("llm crash budget exhausted: {}", self.crash_count),
            };
        }
        MaintenanceAction::NoOp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_machine_capability() {
        let m = LlmMachine::new();
        let cap = m.capability();
        assert_eq!(cap.name, "llm-machine");
        assert_eq!(cap.governance.crash_budget, 10);
        assert!(cap
            .requirements
            .iter()
            .any(|r| matches!(r, Requirement::PortAvailable(15115))));
    }

    #[test]
    fn llm_machine_operate_returns_handle() {
        let m = LlmMachine::new();
        let h = m.operate().expect("operate");
        assert_eq!(h.model, "phi-4-mini-reasoning");
        assert_eq!(h.completions, 0);
    }

    #[test]
    fn chat_request_serializes() {
        let req = ChatRequest {
            model: "phi-4-mini-reasoning".into(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: "hello".into(),
            }],
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("phi-4-mini-reasoning"));
        assert!(json.contains("hello"));
    }

    #[test]
    fn chat_response_deserializes() {
        let json = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1717000000,
            "model": "phi-4-mini-reasoning",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "Hello!"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
        }"#;
        let resp: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.choices[0].message.content, "Hello!");
        assert_eq!(resp.usage.total_tokens, 15);
    }

    #[test]
    fn init_resets_stats() {
        let mut m = LlmMachine::with_config(LlmMachineConfig {
            endpoint: "http://custom:8080/v1".into(),
            ..Default::default()
        });
        m.total_completions = 99;
        m.init(LlmMachineConfig::default()).expect("init resets");
        assert_eq!(m.total_completions, 0);
    }
}
