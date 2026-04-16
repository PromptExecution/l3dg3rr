use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;

use crate::error::McpbError;
use crate::manifest::McpbManifest;

pub struct McpbBundler {
    pub manifest: McpbManifest,
    pub binary_path: PathBuf,
    pub output_path: PathBuf,
}

/// Result of a successful bundle operation.
pub struct BundleArtifact {
    pub path: PathBuf,
    /// Hex-encoded SHA-256 of the .mcpb file (for MCP Registry submission).
    pub sha256: String,
    pub size_bytes: u64,
}

impl McpbBundler {
    pub fn new(manifest: McpbManifest, binary_path: PathBuf, output_path: PathBuf) -> Self {
        Self { manifest, binary_path, output_path }
    }

    /// Assemble a deterministic `.mcpb` ZIP bundle.
    ///
    /// Determinism guarantees:
    /// - Fixed ZIP entry timestamp: 1980-01-01 00:00:00 (ZIP epoch minimum)
    /// - Fixed entry order: manifest.json first, then binary
    /// - Deflate compression
    pub fn bundle(&self) -> Result<BundleArtifact, McpbError> {
        self.manifest.validate()?;

        if !self.binary_path.exists() {
            return Err(McpbError::BinaryNotFound { path: self.binary_path.clone() });
        }

        if let Some(parent) = self.output_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let file = fs::File::create(&self.output_path)?;
        let mut zip = zip::ZipWriter::new(file);

        // Fixed timestamp for reproducibility (1980-01-01 is the ZIP epoch minimum).
        let dt = zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
            .map_err(|e| McpbError::InvalidManifest(format!("ZIP timestamp: {e:?}")))?;

        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .last_modified_time(dt);

        // 1. manifest.json — always first entry
        let manifest_json = serde_json::to_string_pretty(&self.manifest)?;
        zip.start_file("manifest.json", options)?;
        zip.write_all(manifest_json.as_bytes())?;

        // 2. binary — stored under server/<entry_point> per mcpb spec
        //    (${__dirname}/server/<binary> is how Claude Code resolves the absolute path)
        let bin_options = options.unix_permissions(0o755);
        zip.start_file(&self.manifest.server.entry_point, bin_options)?;
        let mut bin_file = fs::File::open(&self.binary_path)?;
        io::copy(&mut bin_file, &mut zip)?;

        zip.finish()?;

        // Compute SHA-256 of the final .mcpb file
        let bytes = fs::read(&self.output_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha256 = hex::encode(hasher.finalize());
        let size_bytes = bytes.len() as u64;

        Ok(BundleArtifact { path: self.output_path.clone(), sha256, size_bytes })
    }
}
