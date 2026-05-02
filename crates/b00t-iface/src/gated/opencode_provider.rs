//! opencode provider surface — formal type representation of an opencode LLM provider.
//!
//! Maps directly to the `provider` section in `opencode.json`.
//! https://opencode.ai/docs/providers/
//!
//! A provider is any OpenAI-compatible `baseURL` + model catalog.
//! This module encodes the schema as a typed b00t surface, so that
//! b00t can manage, validate, and lifecycle-check provider configs.
//!
//! cfg: #[cfg(feature = "b00t")] — requires b00t-native datum types.

use serde::{Deserialize, Serialize};

/// An opencode provider configuration.
///
/// This is the canonical shape. It maps to the `provider.{name}` block
/// in `opencode.json`. Every b00t-managed provider is an instance of this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpencodeProvider {
    /// Provider identifier (e.g. "ollama", "cefprovider", "openrouter").
    pub id: String,
    /// npm package used by opencode to speak to this provider.
    /// Default is `@ai-sdk/openai-compatible` for any OpenAI-compatible API.
    #[serde(default = "default_npm")]
    pub npm: String,
    /// Human-readable display name.
    pub name: String,
    /// Base URL for the API (e.g. "http://localhost:11434/v1").
    pub base_url: String,
    /// Available models keyed by model ID.
    #[serde(default)]
    pub models: Vec<OpencodeModel>,
    /// Environment variables required for authentication.
    #[serde(default)]
    pub required_env: Vec<String>,
}

fn default_npm() -> String {
    "@ai-sdk/openai-compatible".into()
}

/// A single model within an opencode provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpencodeModel {
    /// Model ID as returned by `GET /v1/models`.
    pub id: String,
    /// Display name for the model selection list.
    #[serde(default)]
    pub name: Option<String>,
    /// Context window limit in tokens.
    #[serde(default)]
    pub context_limit: Option<u32>,
    /// Output token limit.
    #[serde(default)]
    pub output_limit: Option<u32>,
}

impl OpencodeProvider {
    /// Validate that the provider config has the minimum required fields.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("provider id must not be empty".into());
        }
        if self.base_url.is_empty() {
            return Err("base_url must not be empty".into());
        }
        if self.models.is_empty() {
            return Err("at least one model must be configured".into());
        }
        for model in &self.models {
            if model.id.is_empty() {
                return Err("model id must not be empty".into());
            }
        }
        Ok(())
    }

    /// Render this provider as a JSON snippet suitable for opencode.json.
    pub fn to_opencode_json(&self) -> serde_json::Value {
        let id = &self.id;
        let npm = &self.npm;
        let name = &self.name;
        let base_url = &self.base_url;

        let models_map: serde_json::Map<String, serde_json::Value> = self
            .models
            .iter()
            .map(|m| {
                let mut entry = serde_json::Map::new();
                if let Some(ref n) = m.name {
                    entry.insert("name".into(), serde_json::Value::String(n.clone()));
                }
                if m.context_limit.is_some() || m.output_limit.is_some() {
                    let mut limit = serde_json::Map::new();
                    if let Some(ctx) = m.context_limit {
                        limit.insert("context".into(), serde_json::Value::Number(ctx.into()));
                    }
                    if let Some(out) = m.output_limit {
                        limit.insert("output".into(), serde_json::Value::Number(out.into()));
                    }
                    entry.insert("limit".into(), serde_json::Value::Object(limit));
                }
                (m.id.clone(), serde_json::Value::Object(entry))
            })
            .collect();

        serde_json::json!({
            id: {
                "npm": npm,
                "name": name,
                "options": {
                    "baseURL": base_url
                },
                "models": models_map
            }
        })
    }
}

/// A collection of providers managed by b00t on this node.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpencodeProviderConfig {
    pub providers: Vec<OpencodeProvider>,
}

impl OpencodeProviderConfig {
    /// Collect all model IDs across all providers.
    pub fn all_model_ids(&self) -> Vec<String> {
        self.providers
            .iter()
            .flat_map(|p| p.models.iter().map(|m| format!("{}/{}", p.id, m.id)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_validate_ok() {
        let p = OpencodeProvider {
            id: "test".into(),
            npm: default_npm(),
            name: "Test Provider".into(),
            base_url: "http://localhost:8080/v1".into(),
            models: vec![OpencodeModel {
                id: "test-model".into(),
                name: Some("Test Model".into()),
                context_limit: Some(32768),
                output_limit: Some(4096),
            }],
            required_env: vec![],
        };
        assert!(p.validate().is_ok());
    }

    #[test]
    fn provider_validate_fails_empty_id() {
        let p = OpencodeProvider {
            id: "".into(),
            npm: default_npm(),
            name: "Bad".into(),
            base_url: "http://localhost:8080/v1".into(),
            models: vec![OpencodeModel {
                id: "m".into(),
                name: None,
                context_limit: None,
                output_limit: None,
            }],
            required_env: vec![],
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn provider_validate_fails_no_models() {
        let p = OpencodeProvider {
            id: "empty".into(),
            npm: default_npm(),
            name: "Empty".into(),
            base_url: "http://localhost:8080/v1".into(),
            models: vec![],
            required_env: vec![],
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn render_to_opencode_json() {
        let p = OpencodeProvider {
            id: "ollama".into(),
            npm: default_npm(),
            name: "Ollama (local)".into(),
            base_url: "http://localhost:11434/v1".into(),
            models: vec![OpencodeModel {
                id: "llama2".into(),
                name: Some("Llama 2".into()),
                context_limit: Some(4096),
                output_limit: None,
            }],
            required_env: vec![],
        };
        let json = p.to_opencode_json();
        let rendered = serde_json::to_string_pretty(&json).unwrap();
        assert!(rendered.contains("ollama"));
        assert!(rendered.contains("baseURL"));
        assert!(rendered.contains("http://localhost:11434/v1"));
    }

    #[test]
    fn collect_model_ids() {
        let cfg = OpencodeProviderConfig {
            providers: vec![
                OpencodeProvider {
                    id: "a".into(),
                    npm: default_npm(),
                    name: "A".into(),
                    base_url: "http://a/v1".into(),
                    models: vec![OpencodeModel {
                        id: "m1".into(),
                        name: None,
                        context_limit: None,
                        output_limit: None,
                    }],
                    required_env: vec![],
                },
                OpencodeProvider {
                    id: "b".into(),
                    npm: default_npm(),
                    name: "B".into(),
                    base_url: "http://b/v1".into(),
                    models: vec![OpencodeModel {
                        id: "m2".into(),
                        name: None,
                        context_limit: None,
                        output_limit: None,
                    }],
                    required_env: vec![],
                },
            ],
        };
        let ids = cfg.all_model_ids();
        assert_eq!(ids, vec!["a/m1".to_string(), "b/m2".to_string()]);
    }
}
