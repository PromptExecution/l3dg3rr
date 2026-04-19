use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::McpbError;

/// Top-level mcpb manifest (spec version 0.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpbManifest {
    pub manifest_version: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: ManifestAuthor,
    pub server: ManifestServer,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<Vec<ConfigField>>,
}

impl McpbManifest {
    pub fn validate(&self) -> Result<(), McpbError> {
        if self.name.is_empty() {
            return Err(McpbError::InvalidManifest("name is empty".into()));
        }
        if self.version.is_empty() {
            return Err(McpbError::InvalidManifest("version is empty".into()));
        }
        if self.manifest_version.is_empty() {
            return Err(McpbError::InvalidManifest(
                "manifest_version is empty".into(),
            ));
        }
        if self.server.entry_point.is_empty() {
            return Err(McpbError::InvalidManifest(
                "server.entry_point is empty".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestAuthor {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestServer {
    #[serde(rename = "type")]
    pub server_type: ServerType,
    pub entry_point: String,
    pub mcp_config: McpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerType {
    Binary,
    Node,
    Python,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub name: String,
    pub description: String,
    pub required: bool,
    #[serde(rename = "type")]
    pub field_type: ConfigFieldType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigFieldType {
    String,
    Boolean,
    Number,
}
