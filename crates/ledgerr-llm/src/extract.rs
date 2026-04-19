use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Structured data extracted from a receipt image via LLM vision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ReceiptExtraction {
    pub vendor_name: Option<String>,
    pub date: Option<String>, // YYYY-MM-DD
    pub total_amount: Option<Decimal>,
    pub currency: Option<String>, // ISO-4217 code
    pub subtotal: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub line_items: Vec<ReceiptLineItem>,
    pub suggested_category: Option<String>,
    pub suggested_tags: Vec<String>,
    /// 0.0–1.0; how confident the model is in the extraction.
    pub confidence: f64,
    /// Raw text found in the image (for audit trail).
    pub raw_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceiptLineItem {
    pub description: String,
    pub quantity: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub amount: Option<Decimal>,
}

/// Extracted metadata from a generic document (invoice, bank statement, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DocumentExtraction {
    pub doc_type_guess: Option<String>,
    pub vendor_or_issuer: Option<String>,
    pub date: Option<String>,
    pub reference_number: Option<String>,
    pub amounts: Vec<ExtractedAmount>,
    pub suggested_tags: Vec<String>,
    pub summary: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedAmount {
    pub label: String,
    pub amount: Decimal,
    pub currency: Option<String>,
}

/// Classification hint returned when the LLM enriches a transaction description.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionClassification {
    pub category: String,
    pub sub_category: Option<String>,
    pub confidence: f64,
    pub reasoning: Option<String>,
    pub suggested_tags: Vec<String>,
}

// ── Prompt templates ──────────────────────────────────────────────────────────

pub(crate) const RECEIPT_SYSTEM_PROMPT: &str = "\
You are a financial document extraction assistant. Extract structured data from receipt images.
Return ONLY valid JSON matching this schema (no markdown, no explanation):
{
  \"vendor_name\": string | null,
  \"date\": \"YYYY-MM-DD\" | null,
  \"total_amount\": number | null,
  \"currency\": \"USD\" | null,
  \"subtotal\": number | null,
  \"tax_amount\": number | null,
  \"line_items\": [{\"description\": string, \"quantity\": number | null, \"unit_price\": number | null, \"amount\": number | null}],
  \"suggested_category\": string | null,
  \"suggested_tags\": [\"#tag\"],
  \"confidence\": 0.0-1.0,
  \"raw_text\": string | null
}";

pub(crate) const DOCUMENT_SYSTEM_PROMPT: &str = "\
You are a financial document classification assistant. Analyze document images and extract key metadata.
Return ONLY valid JSON matching this schema (no markdown, no explanation):
{
  \"doc_type_guess\": \"receipt\" | \"invoice\" | \"bank_statement\" | \"contract\" | \"other\" | null,
  \"vendor_or_issuer\": string | null,
  \"date\": \"YYYY-MM-DD\" | null,
  \"reference_number\": string | null,
  \"amounts\": [{\"label\": string, \"amount\": number, \"currency\": string | null}],
  \"suggested_tags\": [\"#tag\"],
  \"summary\": string | null,
  \"confidence\": 0.0-1.0
}";

pub(crate) const CLASSIFY_SYSTEM_PROMPT: &str = "\
You are a financial transaction categorization assistant. Categorize this transaction for tax/accounting purposes.
Return ONLY valid JSON matching this schema (no markdown, no explanation):
{
  \"category\": string,
  \"sub_category\": string | null,
  \"confidence\": 0.0-1.0,
  \"reasoning\": string | null,
  \"suggested_tags\": [\"#tag\"]
}
Category examples: Meals, Travel, Office Supplies, Software, Contractor, Payroll, Rent, Utilities, Marketing, Other.";
