//! b00t datum schema — foreign-variant invariants for the l3dg3rr node.
//!
//! This crate validates that `.datum` files from the `_b00t_/datums/`
//! symlink exist and are well-formed markdown. b00t datums are freeform
//! markdown documents with `#` headers and `##` sections — not structured
//! serialization formats.
//!
//! Invariants enforced:
//!   - Every datum file listed in `EXPECTED_DATUMS` exists and is non-empty.
//!   - Each datum starts with a `# ` H1 header.

use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum DatumError {
    #[error("I/O error reading datum file '{path}': {source}")]
    Io { path: String, source: std::io::Error },
    #[error("datum file '{path}' is empty")]
    Empty { path: String },
    #[error("datum file '{path}' has no H1 header (line must start with '# ')")]
    NoH1Header { path: String },
}

/// A verified datum: exists, non-empty, has H1 header.
#[derive(Debug, Clone)]
pub struct Datum {
    pub name: String,
    pub path: String,
    pub h1: String,
    pub line_count: usize,
}

/// Read a datum file and validate its structure.
pub fn load_datum(base: &Path, name: &str) -> Result<Datum, DatumError> {
    let path = base.join(format!("{name}.datum"));
    let path_str = path.display().to_string();
    let content = std::fs::read_to_string(&path)
        .map_err(|e| DatumError::Io { path: path_str.clone(), source: e })?;

    if content.trim().is_empty() {
        return Err(DatumError::Empty { path: path_str });
    }

    let h1 = content
        .lines()
        .find(|line| line.starts_with("# ") && !line.starts_with("##"))
        .ok_or_else(|| DatumError::NoH1Header { path: path_str.clone() })?;

    Ok(Datum {
        name: name.to_owned(),
        path: path_str,
        h1: h1.trim_start_matches("# ").to_owned(),
        line_count: content.lines().count(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_all_expected_datums() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../_b00t_/datums");

        for name in &["opencode-codebase-memory-integration", "opencode", "b00t-opencode-gaps", "openagents-control"] {
            let datum = load_datum(&base, name)
                .unwrap_or_else(|e| panic!("datum {name}: {e}"));
            assert!(!datum.h1.is_empty(), "datum {name} has empty H1");
            assert!(datum.line_count > 3, "datum {name} too short ({})", datum.line_count);
        }
    }

    #[test]
    fn load_nonexistent_fails() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../_b00t_/datums");
        let err = load_datum(&base, "does-not-exist").unwrap_err();
        assert!(matches!(err, DatumError::Io { .. }));
    }

    #[test]
    fn symlink_resolves() {
        let symlink = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../_b00t_");
        assert!(symlink.exists(), "_b00t_ symlink must exist at project root");
        assert!(
            symlink.join("datums").exists(),
            "_b00t_/datums must be accessible via symlink"
        );
    }
}
