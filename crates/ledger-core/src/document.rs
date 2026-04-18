use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::tags::Tag;

/// Every format ledgerr can ingest or reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocType {
    Pdf,
    ImageJpeg,
    ImagePng,
    ImageGif,
    ImageWebp,
    ImageTiff,
    SpreadsheetCsv,
    SpreadsheetXlsx,
    Receipt,       // any image treated as a receipt
    BankStatement, // structured document with transactions
    Invoice,
    Other(String),
}

impl DocType {
    /// Infer from file extension; callers may override with magic-byte detection.
    pub fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref()
        {
            Some("pdf") => Self::Pdf,
            Some("jpg") | Some("jpeg") => Self::ImageJpeg,
            Some("png") => Self::ImagePng,
            Some("gif") => Self::ImageGif,
            Some("webp") => Self::ImageWebp,
            Some("tif") | Some("tiff") => Self::ImageTiff,
            Some("csv") => Self::SpreadsheetCsv,
            Some("xlsx") | Some("xls") | Some("xlsm") => Self::SpreadsheetXlsx,
            Some(ext) => Self::Other(ext.to_string()),
            None => Self::Other(String::new()),
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(
            self,
            Self::ImageJpeg
                | Self::ImagePng
                | Self::ImageGif
                | Self::ImageWebp
                | Self::ImageTiff
                | Self::Receipt
        )
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::ImageJpeg => "image/jpeg",
            Self::ImagePng => "image/png",
            Self::ImageGif => "image/gif",
            Self::ImageWebp => "image/webp",
            Self::ImageTiff => "image/tiff",
            Self::SpreadsheetCsv => "text/csv",
            Self::SpreadsheetXlsx => {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }
            Self::Receipt => "image/jpeg",
            Self::BankStatement => "application/pdf",
            Self::Invoice => "application/pdf",
            Self::Other(_) => "application/octet-stream",
        }
    }
}

/// Which Xero entity a document or transaction is linked to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XeroEntityType {
    Contact,
    Account,
    BankAccount,
    Invoice,
    BankTransaction,
}

/// A resolved link from a local document/transaction to a Xero entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XeroLink {
    pub entity_type: XeroEntityType,
    pub xero_id: String,
    pub display_name: String,
    pub linked_at: String, // ISO-8601
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    #[default]
    Pending,
    Processing,
    Indexed,
    Reviewed,
    Archived,
    Error(String),
}

/// The canonical document record stored in ledgerr's document registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentRecord {
    /// blake3 hash of file content — stable, content-addressed identity.
    pub doc_id: String,
    pub file_path: String,
    pub file_name: String,
    pub doc_type: DocType,
    pub tags: Vec<Tag>,
    pub xero_links: Vec<XeroLink>,
    pub status: DocumentStatus,
    /// ISO-8601 timestamp when this document was first indexed.
    pub indexed_at: Option<String>,
    /// Arbitrary key/value properties (OCR output, vendor name, amounts, etc.)
    pub metadata: BTreeMap<String, serde_json::Value>,
}

impl DocumentRecord {
    pub fn new(
        doc_id: impl Into<String>,
        file_path: impl Into<String>,
        doc_type: DocType,
    ) -> Self {
        let file_path = file_path.into();
        let file_name = Path::new(&file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&file_path)
            .to_string();
        Self {
            doc_id: doc_id.into(),
            file_path,
            file_name,
            doc_type,
            tags: Vec::new(),
            xero_links: Vec::new(),
            status: DocumentStatus::Pending,
            indexed_at: None,
            metadata: BTreeMap::new(),
        }
    }

    pub fn add_tag(&mut self, tag: Tag) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, raw: &str) {
        let target = raw.to_ascii_lowercase();
        self.tags.retain(|t| t.as_str() != target);
    }

    pub fn add_xero_link(&mut self, link: XeroLink) {
        let key = (&link.entity_type, &link.xero_id);
        if !self
            .xero_links
            .iter()
            .any(|l| (&l.entity_type, &l.xero_id) == key)
        {
            self.xero_links.push(link);
        }
    }
}

/// Deterministic document ID from file bytes.
pub fn document_id_from_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Deterministic document ID from file path (reads file; returns Err on I/O failure).
pub fn document_id_from_path(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(document_id_from_bytes(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_type_from_extension() {
        assert_eq!(DocType::from_path(Path::new("receipt.jpg")), DocType::ImageJpeg);
        assert_eq!(DocType::from_path(Path::new("statement.pdf")), DocType::Pdf);
        assert_eq!(DocType::from_path(Path::new("data.csv")), DocType::SpreadsheetCsv);
        assert_eq!(DocType::from_path(Path::new("unknown.bin")), DocType::Other("bin".into()));
    }

    #[test]
    fn document_record_tag_dedup() {
        use crate::tags::Tag;
        let mut doc = DocumentRecord::new("abc123", "/tmp/test.pdf", DocType::Pdf);
        let tag = Tag::new("#receipt").unwrap();
        doc.add_tag(tag.clone());
        doc.add_tag(tag);
        assert_eq!(doc.tags.len(), 1);
    }
}
