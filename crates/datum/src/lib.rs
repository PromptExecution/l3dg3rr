#![recursion_limit = "256"]

pub mod ast;
pub mod logic;
pub mod protocol;
pub mod tomllmd;

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

#[derive(Debug, Clone)]
pub struct Datum {
    pub name: String,
    pub path: String,
    pub h1: String,
    pub line_count: usize,
}

/// Read a datum file's content as a string (shared by load_datum and parse_datum_file).
pub fn read_datum_content(base: &Path, name: &str) -> Result<String, DatumError> {
    let path = base.join(format!("{name}.datum"));
    let path_str = path.display().to_string();
    std::fs::read_to_string(&path).map_err(|e| DatumError::Io {
        path: path_str,
        source: e,
    })
}

pub fn load_datum(base: &Path, name: &str) -> Result<Datum, DatumError> {
    let path = base.join(format!("{name}.datum"));
    let path_str = path.display().to_string();
    let content = std::fs::read_to_string(&path).map_err(|e| DatumError::Io {
        path: path_str.clone(),
        source: e,
    })?;

    if content.trim().is_empty() {
        return Err(DatumError::Empty { path: path_str });
    }

    let h1 = content
        .lines()
        .find(|line| line.starts_with("# ") && !line.starts_with("##"))
        .ok_or_else(|| DatumError::NoH1Header {
            path: path_str.clone(),
        })?;

    Ok(Datum {
        name: name.to_owned(),
        path: path_str,
        h1: h1.trim_start_matches("# ").to_owned(),
        line_count: content.lines().count(),
    })
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "real_datums")]
    use super::*;

    #[cfg(feature = "real_datums")]
    fn datum_base() -> std::path::PathBuf {
        // CARGO_MANIFEST_DIR is <repo>/crates/datum
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = crate_dir.join("..").join("..");
        let repo_root = std::fs::canonicalize(&repo_root).unwrap_or(repo_root);

        // Try resolved _b00t_ symlink first
        let b00t_target = repo_root.join("_b00t_");
        if b00t_target.is_dir() {
            let datums = b00t_target.join("datums");
            if datums.exists() {
                return datums;
            }
        }

        // _b00t_ is a text file containing a relative path like "../_b00t_"
        if b00t_target.is_file() {
            if let Ok(target) = std::fs::read_to_string(&b00t_target) {
                let target = target.trim();
                let path = repo_root.join(target).join("datums");
                if path.exists() {
                    return std::fs::canonicalize(&path).unwrap_or(path);
                }
            }
        }

        // Direct fallback for sibling _b00t_ repo
        let sibling = repo_root.join("..").join("_b00t_").join("datums");
        if sibling.exists() {
            return std::fs::canonicalize(&sibling).unwrap_or(sibling);
        }

        repo_root.join("_b00t_").join("datums")
    }

    #[test]
    #[cfg(feature = "real_datums")]
    fn load_all_expected_datums() {
        let base = datum_base();
        for name in &["opencode-codebase-memory-integration", "opencode", "b00t-opencode-gaps", "openagents-control"] {
            let datum = load_datum(&base, name)
                .unwrap_or_else(|e| panic!("datum {name}: {e}"));
            assert!(!datum.h1.is_empty(), "datum {name} has empty H1");
            assert!(datum.line_count > 3, "datum {name} too short ({})", datum.line_count);
        }
    }

    #[test]
    #[cfg(feature = "real_datums")]
    fn load_nonexistent_fails() {
        let base = datum_base();
        let err = load_datum(&base, "does-not-exist").unwrap_err();
        assert!(matches!(err, DatumError::Io { .. }));
    }

    #[test]
    #[cfg(feature = "real_datums")]
    fn datum_base_resolves() {
        let base = datum_base();
        assert!(base.exists(), "datum base must exist: {}", base.display());
    }
}
