use thiserror::Error;

use crate::agent_runtime::{
    AgentRuntime, AgentRuntimeError, ModelRequest, ModelRole, ModelTurn, RigAgentRuntime,
};
use crate::settings::ChatSettings;

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
}
