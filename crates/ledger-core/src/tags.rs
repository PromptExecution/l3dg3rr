use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TagError {
    #[error("tag must start with '#', got: {0}")]
    MissingHash(String),
    #[error("tag is empty after the '#'")]
    Empty,
    #[error("tag contains invalid characters (only lowercase letters, digits, hyphens allowed): {0}")]
    InvalidChars(String),
}

/// A validated workflow tag in `#kebab-case` form.
///
/// Tags are used to annotate documents, transactions, and ontology entities
/// with workflow state, review status, and Xero linkage indicators.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tag(String);

impl Tag {
    /// Create and validate a tag. Input is normalized to `#lowercase-with-hyphens`.
    pub fn new(raw: &str) -> Result<Self, TagError> {
        let s = raw.trim();
        if !s.starts_with('#') {
            return Err(TagError::MissingHash(raw.to_string()));
        }
        let body = &s[1..];
        if body.is_empty() {
            return Err(TagError::Empty);
        }
        let normalized = body.to_ascii_lowercase().replace(' ', "-");
        if !normalized
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err(TagError::InvalidChars(raw.to_string()));
        }
        Ok(Self(format!("#{normalized}")))
    }

    fn sanitize_body(body: &str) -> String {
        let mut sanitized = String::new();
        let mut last_was_hyphen = false;

        for c in body.trim().chars() {
            if c.is_ascii_alphanumeric() {
                sanitized.push(c.to_ascii_lowercase());
                last_was_hyphen = false;
            } else if !sanitized.is_empty() && !last_was_hyphen {
                sanitized.push('-');
                last_was_hyphen = true;
            }
        }

        let sanitized = sanitized.trim_matches('-').to_string();
        if sanitized.is_empty() {
            "tag".to_string()
        } else {
            sanitized
        }
    }

    /// Infallible parse that prefixes '#' if missing and normalizes.
    pub fn normalize(raw: &str) -> Self {
        let s = raw.trim();
        let s = if s.starts_with('#') {
            s.to_string()
        } else {
            format!("#{s}")
        };
        Self::new(&s).unwrap_or_else(|_| {
            let body = s.strip_prefix('#').unwrap_or(&s);
            Self(format!("#{}", Self::sanitize_body(body)))
        })
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ── Well-known workflow tags ──────────────────────────────────────────────────

pub const TAG_PENDING_REVIEW: &str = "#pending-review";
pub const TAG_XERO_LINKED: &str = "#xero-linked";
pub const TAG_RECEIPT: &str = "#receipt";
pub const TAG_INVOICE: &str = "#invoice";
pub const TAG_BANK_STATEMENT: &str = "#bank-statement";
pub const TAG_FLAGGED: &str = "#flagged";
pub const TAG_APPROVED: &str = "#approved";
pub const TAG_ARCHIVED: &str = "#archived";
pub const TAG_OCR_COMPLETE: &str = "#ocr-complete";
pub const TAG_NEEDS_AMOUNT: &str = "#needs-amount";
pub const TAG_DUPLICATE: &str = "#duplicate";

/// Parse a slice of raw tag strings into validated Tags, collecting errors.
pub fn parse_tags(raws: &[impl AsRef<str>]) -> (Vec<Tag>, Vec<TagError>) {
    let mut tags = Vec::new();
    let mut errors = Vec::new();
    for r in raws {
        match Tag::new(r.as_ref()) {
            Ok(t) => tags.push(t),
            Err(e) => errors.push(e),
        }
    }
    (tags, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_tag_roundtrip() {
        let t = Tag::new("#pending-review").unwrap();
        assert_eq!(t.as_str(), "#pending-review");
    }

    #[test]
    fn tag_normalizes_spaces() {
        let t = Tag::new("#needs review").unwrap();
        assert_eq!(t.as_str(), "#needs-review");
    }

    #[test]
    fn tag_normalizes_uppercase() {
        let t = Tag::new("#XeroLinked").unwrap();
        assert_eq!(t.as_str(), "#xerolinked");
    }

    #[test]
    fn tag_requires_hash() {
        assert!(Tag::new("no-hash").is_err());
    }

    #[test]
    fn tag_empty_body_rejected() {
        assert!(Tag::new("#").is_err());
    }

    #[test]
    fn normalize_without_hash() {
        let t = Tag::normalize("receipt");
        assert_eq!(t.as_str(), "#receipt");
    }
}
