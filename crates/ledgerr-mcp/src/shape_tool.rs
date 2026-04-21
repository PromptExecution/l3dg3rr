//! Document shape tool: classify document type and vendor from filename and sample content.

use serde::{Deserialize, Serialize};

use ledger_core::document::DocType;
use ledger_core::document_shape::DocumentShape;

/// Request to classify a document's shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentShapeRequest {
    /// Filename of the document (e.g., "CHASE--CHECKING--2024-01--STATEMENT.PDF").
    pub filename: String,
    /// Sample of document content (first ~2 KB, typically from Docling extraction).
    pub sample_content: String,
}

/// Classify a document's shape from filename and sample content.
///
/// Calls `classify_document_shape()` from `ledger_core::document_shape`.
/// Infers the vendor, account type, statement format, date format, currency,
/// and confidence level from filename convention, keyword matching, and content analysis.
pub fn get_document_shape(req: GetDocumentShapeRequest) -> DocumentShape {
    // Use a default DocType of Pdf for all requests; the actual type inference
    // could be enhanced based on filename extension if needed.
    let doc_type = DocType::Pdf;
    ledger_core::document_shape::classify_document_shape(
        &doc_type,
        &req.filename,
        &req.sample_content,
    )
}
