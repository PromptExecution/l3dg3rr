/// Filesystem metadata sync: read/write ledgerr metadata alongside files.
///
/// On every platform, a `.{filename}.ledgerr.json` sidecar is the canonical
/// backend. An xattr backend is compiled in on Linux for tools that prefer
/// in-band metadata (file managers, indexers).
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsMetaError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[cfg(target_os = "linux")]
    #[error("xattr error: {0}")]
    Xattr(String),
}

/// The metadata ledgerr attaches to every managed file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FsMetadata {
    /// blake3 content hash — ties the filesystem file back to the DocumentRecord.
    pub doc_id: String,
    /// Validated workflow tags (e.g. `["#receipt", "#xero-linked"]`).
    pub tags: Vec<String>,
    pub xero_contact_id: Option<String>,
    pub xero_account_id: Option<String>,
    /// Current processing status (pending / indexed / archived / …).
    pub status: String,
    /// ISO-8601 timestamp of first ledgerr index.
    pub indexed_at: Option<String>,
}

pub trait MetadataBackend: Send + Sync {
    fn read(&self, path: &Path) -> Result<Option<FsMetadata>, FsMetaError>;
    fn write(&self, path: &Path, meta: &FsMetadata) -> Result<(), FsMetaError>;
    fn remove(&self, path: &Path) -> Result<(), FsMetaError>;
}

// ── Sidecar backend (universal) ───────────────────────────────────────────────

fn sidecar_path(file: &Path) -> PathBuf {
    let name = file
        .file_name()
        .map(|n| format!(".{}.ledgerr.json", n.to_string_lossy()))
        .unwrap_or_else(|| ".ledgerr.json".into());
    file.with_file_name(name)
}

pub struct SidecarBackend;

impl MetadataBackend for SidecarBackend {
    fn read(&self, path: &Path) -> Result<Option<FsMetadata>, FsMetaError> {
        let sp = sidecar_path(path);
        if !sp.exists() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(&sp)?;
        Ok(Some(serde_json::from_str(&raw)?))
    }

    fn write(&self, path: &Path, meta: &FsMetadata) -> Result<(), FsMetaError> {
        let sp = sidecar_path(path);
        let json = serde_json::to_string_pretty(meta)?;
        std::fs::write(sp, json)?;
        Ok(())
    }

    fn remove(&self, path: &Path) -> Result<(), FsMetaError> {
        let sp = sidecar_path(path);
        if sp.exists() {
            std::fs::remove_file(sp)?;
        }
        Ok(())
    }
}

// ── xattr backend (Linux only) ────────────────────────────────────────────────

#[cfg(target_os = "linux")]
pub struct XattrBackend;

#[cfg(target_os = "linux")]
const XATTR_KEY: &str = "user.ledgerr.meta";

#[cfg(target_os = "linux")]
impl MetadataBackend for XattrBackend {
    fn read(&self, path: &Path) -> Result<Option<FsMetadata>, FsMetaError> {
        match xattr::get(path, XATTR_KEY) {
            Ok(Some(bytes)) => {
                let meta = serde_json::from_slice(&bytes)?;
                Ok(Some(meta))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(FsMetaError::Xattr(e.to_string())),
        }
    }

    fn write(&self, path: &Path, meta: &FsMetadata) -> Result<(), FsMetaError> {
        let bytes = serde_json::to_vec(meta)?;
        xattr::set(path, XATTR_KEY, &bytes).map_err(|e| FsMetaError::Xattr(e.to_string()))
    }

    fn remove(&self, path: &Path) -> Result<(), FsMetaError> {
        xattr::remove(path, XATTR_KEY).map_err(|e| FsMetaError::Xattr(e.to_string()))
    }
}

/// Build the best available backend for the current platform.
pub fn default_backend() -> Box<dyn MetadataBackend> {
    #[cfg(target_os = "linux")]
    return Box::new(XattrBackend);
    #[cfg(not(target_os = "linux"))]
    Box::new(SidecarBackend)
}

/// Convenience: read metadata using the platform default backend.
pub fn read_meta(path: &Path) -> Result<Option<FsMetadata>, FsMetaError> {
    default_backend().read(path)
}

/// Convenience: write metadata using the platform default backend.
pub fn write_meta(path: &Path, meta: &FsMetadata) -> Result<(), FsMetaError> {
    default_backend().write(path, meta)
}

/// Scan a directory tree and yield (path, metadata) for every file that has
/// a ledgerr sidecar or xattr. Always checks sidecars (universal) and also
/// the platform default backend (xattr on Linux).
pub fn scan_directory(
    dir: &Path,
    recursive: bool,
) -> Result<Vec<(PathBuf, FsMetadata)>, FsMetaError> {
    let mut results = Vec::new();
    scan_dir_inner(dir, recursive, &mut results)?;
    Ok(results)
}

fn scan_dir_inner(
    dir: &Path,
    recursive: bool,
    results: &mut Vec<(PathBuf, FsMetadata)>,
) -> Result<(), FsMetaError> {
    let sidecar = SidecarBackend;
    #[cfg(target_os = "linux")]
    let xattr_backend = XattrBackend;

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        // Skip hidden files, sidecars, and system dirs.
        if name.starts_with('.') || name == "target" || name == ".git" {
            continue;
        }

        if path.is_dir() && recursive {
            scan_dir_inner(&path, recursive, results)?;
        } else if path.is_file() {
            // Try sidecar first (universal), then platform xattr.
            let meta = if let Some(m) = sidecar.read(&path)? {
                Some(m)
            } else {
                #[cfg(target_os = "linux")]
                { xattr_backend.read(&path)? }
                #[cfg(not(target_os = "linux"))]
                { None }
            };
            if let Some(m) = meta {
                results.push((path, m));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn sidecar_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("invoice.pdf");
        std::fs::File::create(&file).unwrap().flush().unwrap();

        let meta = FsMetadata {
            doc_id: "abc123".into(),
            tags: vec!["#receipt".into(), "#pending-review".into()],
            xero_contact_id: Some("xero-contact-001".into()),
            xero_account_id: None,
            status: "indexed".into(),
            indexed_at: Some("2026-04-18T12:00:00Z".into()),
        };

        let backend = SidecarBackend;
        backend.write(&file, &meta).unwrap();

        let loaded = backend.read(&file).unwrap().expect("should exist");
        assert_eq!(loaded, meta);

        backend.remove(&file).unwrap();
        assert!(backend.read(&file).unwrap().is_none());
    }

    #[test]
    fn scan_finds_sidecars() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("statement.pdf");
        std::fs::File::create(&file).unwrap().flush().unwrap();

        let meta = FsMetadata {
            doc_id: "def456".into(),
            status: "pending".into(),
            ..Default::default()
        };
        SidecarBackend.write(&file, &meta).unwrap();

        let found = scan_directory(dir.path(), false).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, file);
    }
}
