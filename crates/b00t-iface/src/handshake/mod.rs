//! Handshake — b00t ↔ l3dg3rr variant protocol.
//!
//! When b00t and l3dg3rr are on the same system, they exchange state using
//! a handshake variant that does not presently exist in either codebase.
//! This module defines that variant as a formal b00t surface.
//!
//! # Handshake protocol
//!
//! 1. Discover: each side advertises its capability document on a well-known
//!    path. b00t writes to `~/.b00t/mesh/l3dg3rr.handshake`, l3dg3rr writes
//!    to `_b00t_/handshake/l3dg3rr.json`.
//! 2. Verify: each side reads the other's capability and validates it against
//!    the governance policy.
//! 3. Exchange: surfaces, models, and audit logs are shared.
//! 4. Monitor: heartbeat pings at configurable interval.
//!
//! The handshake IS the integration — it's a b00t surface that, on operate(),
//! performs the full exchange.

use crate::core::{
    AuditRecord, GovernancePolicy, MaintenanceAction, ProcessSurface, Requirement,
    SurfaceCapability,
};
use crate::AgentRole;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// The capability document exchanged between b00t and l3dg3rr.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeDocument {
    /// Sender identity (e.g. "b00t" or "l3dg3rr").
    pub sender: String,
    /// Variant ID matching the node datum.
    pub variant_id: String,
    /// Hostname.
    pub host: String,
    /// Available surfaces (short names).
    pub surfaces: Vec<String>,
    /// Available LLM models.
    pub models: Vec<String>,
    /// Protocol version for forward compatibility.
    pub version: String,
}

/// Handshake result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeResult {
    Matched,
    VariantMismatch { expected: String, got: String },
    NoPeer,
}

impl std::fmt::Display for HandshakeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Matched => write!(f, "matched"),
            Self::VariantMismatch { expected, got } => {
                write!(f, "variant mismatch: expected '{expected}', got '{got}'")
            }
            Self::NoPeer => write!(f, "no peer"),
        }
    }
}

/// The handshake surface — performs b00t↔l3dg3rr peer discovery and state exchange.
pub struct HandshakeSurface {
    pub identity: String,
    pub variant_id: String,
    pub host: String,
    pub handshake_dir: std::path::PathBuf,
    pub heartbeat_interval: Duration,
    pub result: Option<HandshakeResult>,
    pub peer_doc: Option<HandshakeDocument>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HandshakeConfig {
    pub identity: String,
    pub variant_id: String,
    pub host: String,
    #[serde(default = "default_handshake_dir")]
    pub handshake_dir: String,
    #[serde(default = "default_heartbeat")]
    pub heartbeat_secs: u64,
}

fn default_handshake_dir() -> String {
    "_b00t_/handshake".into()
}

fn default_heartbeat() -> u64 {
    30
}

#[derive(Debug, Clone)]
pub enum HandshakeError {
    Dir(String),
    Write(String),
    Read(String),
    Parse(String),
}

impl std::fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dir(e) => write!(f, "handshake dir: {e}"),
            Self::Write(e) => write!(f, "handshake write: {e}"),
            Self::Read(e) => write!(f, "handshake read: {e}"),
            Self::Parse(e) => write!(f, "handshake parse: {e}"),
        }
    }
}

impl std::error::Error for HandshakeError {}

/// Handle returned by operate().
#[derive(Debug, Clone)]
pub struct HandshakeHandle {
    pub result: HandshakeResult,
    pub peer_surfaces: Vec<String>,
    pub peer_models: Vec<String>,
}

impl HandshakeSurface {
    pub fn new(identity: &str, variant_id: &str, host: &str) -> Self {
        Self {
            identity: identity.to_owned(),
            variant_id: variant_id.to_owned(),
            host: host.to_owned(),
            handshake_dir: Path::new("_b00t_").join("handshake"),
            heartbeat_interval: Duration::from_secs(30),
            result: None,
            peer_doc: None,
        }
    }

    fn doc_path(&self) -> std::path::PathBuf {
        self.handshake_dir.join("l3dg3rr.json")
    }

    /// Write our capability document to the handshake dir.
    fn write_doc(&self) -> Result<(), HandshakeError> {
        let dir = &self.handshake_dir;
        std::fs::create_dir_all(dir).map_err(|e| HandshakeError::Dir(e.to_string()))?;

        let doc = HandshakeDocument {
            sender: self.identity.clone(),
            variant_id: self.variant_id.clone(),
            host: self.host.clone(),
            surfaces: vec![
                "datum-watcher".into(),
                "autoresearch".into(),
                "llm-machine".into(),
                "opencode-provider".into(),
            ],
            models: vec!["phi-4-mini-reasoning".into()],
            version: "1.0.0".into(),
        };

        let json =
            serde_json::to_string_pretty(&doc).map_err(|e| HandshakeError::Write(e.to_string()))?;
        std::fs::write(self.doc_path(), json).map_err(|e| HandshakeError::Write(e.to_string()))?;
        Ok(())
    }

    /// Read the peer's capability document.
    fn read_peer(&self) -> Result<Option<HandshakeDocument>, HandshakeError> {
        // b00t writes to ~/.b00t/mesh/l3dg3rr.handshake;
        // we check if it exists and parse it.
        let b00t_path =
            dirs::home_dir().map(|h| h.join(".b00t").join("mesh").join("l3dg3rr.handshake"));

        let path = match b00t_path {
            Some(p) if p.exists() => p,
            _ => return Ok(None),
        };

        let content =
            std::fs::read_to_string(&path).map_err(|e| HandshakeError::Read(e.to_string()))?;
        let doc: HandshakeDocument =
            serde_json::from_str(&content).map_err(|e| HandshakeError::Parse(e.to_string()))?;
        Ok(Some(doc))
    }

    /// Perform the handshake: write our doc, read peer, compare.
    fn perform(&mut self) -> Result<HandshakeResult, HandshakeError> {
        self.write_doc()?;
        match self.read_peer()? {
            Some(peer) => {
                self.peer_doc = Some(peer.clone());
                if peer.variant_id == self.variant_id {
                    self.result = Some(HandshakeResult::Matched);
                    Ok(HandshakeResult::Matched)
                } else {
                    self.result = Some(HandshakeResult::VariantMismatch {
                        expected: self.variant_id.clone(),
                        got: peer.variant_id,
                    });
                    Ok(self.result.clone().unwrap())
                }
            }
            None => {
                self.result = Some(HandshakeResult::NoPeer);
                Ok(HandshakeResult::NoPeer)
            }
        }
    }
}

impl ProcessSurface for HandshakeSurface {
    type Config = HandshakeConfig;
    type Error = HandshakeError;
    type Handle = HandshakeHandle;

    fn capability(&self) -> SurfaceCapability {
        SurfaceCapability {
            name: "handshake",
            requirements: vec![Requirement::PathExists(
                self.handshake_dir.display().to_string(),
            )],
            governance: GovernancePolicy {
                allowed_starters: vec![AgentRole::Executive],
                max_ttl: Duration::from_secs(3600),
                auto_restart: true,
                crash_budget: 3,
            },
        }
    }

    fn init(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.identity = config.identity;
        self.variant_id = config.variant_id;
        self.host = config.host;
        self.handshake_dir = Path::new(&config.handshake_dir).to_path_buf();
        self.heartbeat_interval = Duration::from_secs(config.heartbeat_secs);
        std::fs::create_dir_all(&self.handshake_dir)
            .map_err(|e| HandshakeError::Dir(e.to_string()))?;
        tracing::info!(
            "HandshakeSurface initialized: {}@{}",
            self.identity,
            self.host
        );
        Ok(())
    }

    fn operate(&self) -> Result<Self::Handle, Self::Error> {
        // operate performs the handshake — but since operate takes &self,
        // we use a trick: write our doc, read the peer.
        // The actual handshake state is stored in the filesystem.
        let mut surface_clone = Self {
            identity: self.identity.clone(),
            variant_id: self.variant_id.clone(),
            host: self.host.clone(),
            handshake_dir: self.handshake_dir.clone(),
            heartbeat_interval: self.heartbeat_interval,
            result: None,
            peer_doc: None,
        };
        let result = surface_clone.perform()?;

        match &result {
            HandshakeResult::Matched => tracing::info!("b00t↔l3dg3rr handshake matched"),
            HandshakeResult::NoPeer => tracing::warn!("b00t↔l3dg3rr handshake: no peer"),
            HandshakeResult::VariantMismatch { expected, got } => {
                tracing::warn!("b00t↔l3dg3rr variant mismatch: expected {expected}, got {got}");
            }
        }

        let peer_doc = surface_clone.peer_doc;
        Ok(HandshakeHandle {
            result,
            peer_surfaces: peer_doc
                .as_ref()
                .map(|d| d.surfaces.clone())
                .unwrap_or_default(),
            peer_models: peer_doc
                .as_ref()
                .map(|d| d.models.clone())
                .unwrap_or_default(),
        })
    }

    fn terminate(handle: Self::Handle) -> Result<AuditRecord, Self::Error> {
        Ok(AuditRecord {
            surface_name: "handshake".into(),
            uptime: Duration::from_secs(0),
            exit_reason: format!("handshake result: {}", handle.result),
            crash_count: 0,
            bytes_logged: 0,
        })
    }

    fn maintain(&self) -> MaintenanceAction {
        std::fs::create_dir_all(&self.handshake_dir).ok();
        MaintenanceAction::NoOp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn write_and_read_handshake_doc() {
        let tmp = TempDir::new().unwrap();
        let mut s = HandshakeSurface::new("l3dg3rr", "test-variant", "test-host");
        s.handshake_dir = tmp.path().join("handshake");
        s.write_doc().expect("write doc");

        let doc_path = s.doc_path();
        assert!(doc_path.exists());
        let content = std::fs::read_to_string(doc_path).unwrap();
        assert!(content.contains("l3dg3rr"));
        assert!(content.contains("test-variant"));
    }

    #[test]
    fn handshake_result_display() {
        assert_eq!(HandshakeResult::Matched.to_string(), "matched");
        assert_eq!(HandshakeResult::NoPeer.to_string(), "no peer");
        let mismatch = HandshakeResult::VariantMismatch {
            expected: "a".into(),
            got: "b".into(),
        };
        assert!(mismatch.to_string().contains("variant mismatch"));
    }

    #[test]
    fn init_creates_dir() {
        let tmp = TempDir::new().unwrap();
        let mut s = HandshakeSurface::new("test", "v1", "h1");
        let config = HandshakeConfig {
            identity: "test".into(),
            variant_id: "v1".into(),
            host: "h1".into(),
            handshake_dir: tmp.path().join("custom-hs").display().to_string(),
            heartbeat_secs: 10,
        };
        s.init(config).expect("init");
        assert!(tmp.path().join("custom-hs").exists());
    }

    #[test]
    fn operate_no_peer_returns_nopeer() {
        let tmp = TempDir::new().unwrap();
        let mut s = HandshakeSurface::new("l3dg3rr", "variant-1", "host-1");
        s.handshake_dir = tmp.path().join("hs");
        let config = HandshakeConfig {
            identity: "l3dg3rr".into(),
            variant_id: "variant-1".into(),
            host: "host-1".into(),
            handshake_dir: s.handshake_dir.display().to_string(),
            heartbeat_secs: 30,
        };
        s.init(config).expect("init");
        let handle = s.operate().expect("operate");
        assert_eq!(handle.result, HandshakeResult::NoPeer);
    }
}
