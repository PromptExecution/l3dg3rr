use std::io::Write;
use std::path::{Path, PathBuf};

use thiserror::Error;

use super::schema::AppSettings;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<AppSettings, SettingsError> {
        if !self.path.exists() {
            return Ok(AppSettings::default());
        }
        let raw = std::fs::read_to_string(&self.path)?;
        match serde_json::from_str::<AppSettings>(&raw) {
            Ok(settings) => Ok(settings),
            Err(_) => Ok(AppSettings::default()),
        }
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), SettingsError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let temp_path = self.path.with_extension("json.tmp");
        let json = serde_json::to_vec_pretty(settings)?;
        let mut temp_file = std::fs::File::create(&temp_path)?;
        temp_file.write_all(&json)?;
        temp_file.flush()?;
        drop(temp_file);

        // On Windows, `std::fs::rename` does not overwrite an existing destination,
        // so we remove the destination first if it exists to keep the swap atomic
        // enough for single-user local operation.
        #[cfg(windows)]
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }

        std::fs::rename(temp_path, &self.path)?;
        Ok(())
    }
}
