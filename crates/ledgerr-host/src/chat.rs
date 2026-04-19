use rig::{
    client::CompletionClient,
    completion::{AssistantContent, CompletionModel, Message},
    providers::openai,
};
use thiserror::Error;

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
    #[error("request client failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("failed to create async runtime: {0}")]
    Runtime(#[from] std::io::Error),
    #[error("chat request failed: {0}")]
    Rig(#[from] rig::completion::CompletionError),
    #[error("chat client setup failed: {0}")]
    RigHttp(#[from] rig::http_client::Error),
    #[error("response did not contain an assistant message")]
    MissingAssistantMessage,
}

pub fn send_chat_message(
    settings: &ChatSettings,
    history: &[ChatTurn],
    pending_message: &str,
) -> Result<String, ChatError> {
    if settings.endpoint_url.trim().is_empty() {
        return Err(ChatError::MissingEndpoint);
    }
    if settings.model.trim().is_empty() {
        return Err(ChatError::MissingModel);
    }
    if settings.api_key.trim().is_empty() {
        return Err(ChatError::MissingApiKey);
    }
    if pending_message.trim().is_empty() {
        return Err(ChatError::EmptyMessage);
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(send_chat_message_async(settings, history, pending_message))
}

async fn send_chat_message_async(
    settings: &ChatSettings,
    history: &[ChatTurn],
    pending_message: &str,
) -> Result<String, ChatError> {
    let client = openai::CompletionsClient::builder()
        .api_key(settings.api_key.trim())
        .base_url(normalize_base_url(settings.endpoint_url.trim()))
        .build()?;
    let model = client.completion_model(settings.model.trim());
    let request = build_request(model.clone(), settings, history, pending_message);
    let response = model.completion(request).await?;
    extract_assistant_message(response.choice.into_iter()).ok_or(ChatError::MissingAssistantMessage)
}

fn build_request<M: CompletionModel>(
    model: M,
    settings: &ChatSettings,
    history: &[ChatTurn],
    pending_message: &str,
) -> rig::completion::CompletionRequest {
    let mut request = model.completion_request(Message::user(pending_message.trim()));

    if !settings.system_prompt.trim().is_empty() {
        request = request.preamble(settings.system_prompt.trim().to_string());
    }

    if !history.is_empty() {
        request = request.messages(history.iter().map(history_message));
    }

    request.build()
}

fn history_message(turn: &ChatTurn) -> Message {
    match turn.role {
        ChatRole::System => Message::system(turn.content.trim()),
        ChatRole::User => Message::user(turn.content.trim()),
        ChatRole::Assistant => Message::assistant(turn.content.trim()),
    }
}

fn normalize_base_url(endpoint_url: &str) -> String {
    endpoint_url
        .trim_end_matches('/')
        .trim_end_matches("/chat/completions")
        .trim_end_matches("/responses")
        .to_string()
}

fn extract_assistant_message(
    contents: impl IntoIterator<Item = AssistantContent>,
) -> Option<String> {
    contents
        .into_iter()
        .find_map(|content| match content {
            AssistantContent::Text(text) => {
                let trimmed = text.text.trim();
                (!trimmed.is_empty()).then(|| trimmed.to_string())
            }
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rig::{completion::AssistantContent, providers::openai};

    fn test_settings() -> ChatSettings {
        ChatSettings {
            endpoint_url: "https://example.test/v1/chat/completions".to_string(),
            api_key: "test-key".to_string(),
            model: "gpt-test".to_string(),
            system_prompt: "You are terse.".to_string(),
        }
    }

    #[test]
    fn build_request_includes_system_history_and_pending_user_message() {
        let history = vec![ChatTurn {
            role: ChatRole::Assistant,
            content: "Earlier answer".to_string(),
        }];

        let model = openai::Client::new("test-key")
            .expect("test client")
            .completions_api()
            .completion_model("gpt-test");
        let request = build_request(model, &test_settings(), &history, "What next?");
        let messages: Vec<_> = request.chat_history.into_iter().collect();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0], Message::system("You are terse."));
        assert_eq!(messages[1], Message::assistant("Earlier answer"));
        assert_eq!(messages[2], Message::user("What next?"));
    }

    #[test]
    fn extract_assistant_message_prefers_text_content() {
        let message = extract_assistant_message([
            AssistantContent::text("  hello world  "),
            AssistantContent::reasoning("ignored"),
        ]);
        assert_eq!(message.as_deref(), Some("hello world"));
    }

    #[test]
    fn normalize_base_url_accepts_full_or_root_openai_urls() {
        assert_eq!(
            normalize_base_url("https://api.openai.com/v1/chat/completions"),
            "https://api.openai.com/v1"
        );
        assert_eq!(
            normalize_base_url("https://api.openai.com/v1/"),
            "https://api.openai.com/v1"
        );
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
