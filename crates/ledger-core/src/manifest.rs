use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub session: Session,
    #[serde(default)]
    pub accounts: BTreeMap<String, AccountDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Session {
    pub workbook_path: String,
    pub active_year: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountDef {
    pub institution: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub currency: String,
}

impl Manifest {
    pub fn parse(src: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(src)
    }

    pub fn list_account_ids(&self) -> Vec<String> {
        self.accounts.keys().cloned().collect()
    }
}
