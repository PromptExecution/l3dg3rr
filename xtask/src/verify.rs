use std::io::Read;
use std::path::Path;

use crate::error::McpbError;
use crate::manifest::McpbManifest;

/// Validate a `.mcpb` ZIP bundle:
/// - Must be a valid ZIP archive
/// - Must contain `manifest.json` parseable as [`McpbManifest`]
/// - Manifest must pass validation
/// - The declared `server.entry_point` binary must exist as a ZIP entry
pub fn verify_bundle(path: &Path) -> Result<McpbManifest, McpbError> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut manifest_bytes = Vec::new();
    {
        let mut entry = archive
            .by_name("manifest.json")
            .map_err(|_| McpbError::InvalidManifest("missing manifest.json entry".into()))?;
        entry.read_to_end(&mut manifest_bytes)?;
    }

    let manifest: McpbManifest = serde_json::from_slice(&manifest_bytes)?;
    manifest.validate()?;

    // Verify the declared entry_point is present in the archive
    archive.by_name(&manifest.server.entry_point).map_err(|_| {
        McpbError::InvalidManifest(format!(
            "declared entry_point '{}' not found in bundle",
            manifest.server.entry_point
        ))
    })?;

    Ok(manifest)
}
