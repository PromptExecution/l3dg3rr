use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::McpbError;

/// Typed representation of the MCP Registry `server.json` (schema 2025-12-11).
/// This is the file that `mcp-publisher publish` submits to the registry.
/// It is distinct from the `.mcpb` bundle's `manifest.json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerJson {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub description: String,
    pub repository: Repository,
    pub version: String,
    pub packages: Vec<Package>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub url: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    #[serde(rename = "registryType")]
    pub registry_type: String,
    pub identifier: String,
    /// Required for `registryType: "mcpb"`. SHA-256 hex of the .mcpb file.
    /// MCP clients validate this before installation.
    #[serde(rename = "fileSha256", skip_serializing_if = "Option::is_none")]
    pub file_sha256: Option<String>,
    pub transport: Transport,
    #[serde(
        rename = "environmentVariables",
        skip_serializing_if = "Option::is_none"
    )]
    pub environment_variables: Option<Vec<EnvVar>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transport {
    #[serde(rename = "type")]
    pub transport_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub description: String,
    #[serde(rename = "isRequired")]
    pub is_required: bool,
    pub format: String,
    #[serde(rename = "isSecret")]
    pub is_secret: bool,
}

impl ServerJson {
    pub fn load(path: &Path) -> Result<Self, McpbError> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self, path: &Path) -> Result<(), McpbError> {
        let content = serde_json::to_string_pretty(self)?;
        // Trailing newline — keeps git diffs clean
        std::fs::write(path, format!("{content}\n"))?;
        Ok(())
    }

    /// Update version, mcpb package identifier URL, and fileSha256 in-place.
    /// Targets the first `registryType: "mcpb"` entry in `packages`.
    pub fn update_mcpb(
        &mut self,
        version: &str,
        identifier: &str,
        sha256: &str,
    ) -> Result<(), McpbError> {
        self.version = version.to_string();

        let pkg = self
            .packages
            .iter_mut()
            .find(|p| p.registry_type == "mcpb")
            .ok_or_else(|| {
                McpbError::InvalidManifest("no mcpb package entry found in server.json".into())
            })?;

        pkg.identifier = identifier.to_string();
        pkg.file_sha256 = Some(sha256.to_string());
        Ok(())
    }
}
