use std::path::Path;
use std::process::Command;

use crate::error::McpbError;

/// Uploads a `.mcpb` artifact to a GitHub release using the `gh` CLI.
pub struct GitHubPublisher {
    pub release_tag: String,
    pub repo: Option<String>,
}

impl GitHubPublisher {
    pub fn new(release_tag: impl Into<String>) -> Self {
        Self { release_tag: release_tag.into(), repo: None }
    }

    pub fn with_repo(mut self, repo: impl Into<String>) -> Self {
        self.repo = Some(repo.into());
        self
    }

    pub fn upload(&self, artifact_path: &Path) -> Result<(), McpbError> {
        let mut cmd = Command::new("gh");
        cmd.args(["release", "upload", &self.release_tag]);
        cmd.arg(artifact_path);
        cmd.arg("--clobber"); // idempotent re-runs
        if let Some(repo) = &self.repo {
            cmd.args(["-R", repo]);
        }

        let output = cmd.output().map_err(|e| {
            McpbError::PublishFailed(format!("gh CLI not found: {e}"))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpbError::PublishFailed(format!(
                "gh release upload failed: {stderr}"
            )));
        }

        Ok(())
    }
}

/// Generates and submits the MCP Registry server entry via `mcp-publisher` CLI.
///
/// This is a stub until OIDC/token auth is configured for the MCP Registry.
/// `publish_json` is public so CI can inspect the submission payload before executing.
pub struct McpRegistryPublisher {
    pub release_tag: String,
    pub server_name: String,
    pub description: String,
}

impl McpRegistryPublisher {
    pub fn new(
        release_tag: impl Into<String>,
        server_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            release_tag: release_tag.into(),
            server_name: server_name.into(),
            description: description.into(),
        }
    }

    /// Returns the server.json payload required by mcp-publisher.
    pub fn publish_json(&self, artifact_url: &str, sha256: &str) -> serde_json::Value {
        serde_json::json!({
            "name": self.server_name,
            "description": self.description,
            "version": self.release_tag,
            "packages": [{
                "registryType": "mcpb",
                "identifier": artifact_url,
                "fileSha256": sha256
            }]
        })
    }

    /// Executes `mcp-publisher publish server.json`. Requires `mcp-publisher` on PATH
    /// and a valid auth token (GitHub OIDC in CI, or interactive login locally).
    pub fn publish(&self, artifact_url: &str, sha256: &str) -> Result<(), McpbError> {
        let payload = serde_json::to_string_pretty(&self.publish_json(artifact_url, sha256))?;

        let tmp = std::env::temp_dir().join("mcp-server-publish.json");
        std::fs::write(&tmp, payload.as_bytes())?;

        let output = Command::new("mcp-publisher")
            .args(["publish", tmp.to_str().unwrap_or("mcp-server-publish.json")])
            .output()
            .map_err(|e| McpbError::PublishFailed(
                format!("mcp-publisher not available (install from MCP Registry releases): {e}")
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpbError::PublishFailed(format!(
                "mcp-publisher publish failed: {stderr}"
            )));
        }

        Ok(())
    }
}
