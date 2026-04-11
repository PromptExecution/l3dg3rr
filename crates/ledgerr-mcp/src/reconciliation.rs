use std::str::FromStr;

use rust_decimal::Decimal;

use crate::ToolError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationStageRequest {
    pub source_total: String,
    pub extracted_total: String,
    pub posting_amounts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationDiagnostic {
    pub key: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationStageResponse {
    pub stage: String,
    pub status: String,
    pub blocked_reasons: Vec<String>,
    pub stage_marker: String,
    pub diagnostics: Vec<ReconciliationDiagnostic>,
}

pub fn validate_stage(
    request: &ReconciliationStageRequest,
) -> Result<ReconciliationStageResponse, ToolError> {
    parse_request_decimals(request)?;
    Ok(ReconciliationStageResponse {
        stage: "validate".to_string(),
        status: "passed".to_string(),
        blocked_reasons: Vec::new(),
        stage_marker: "validate:passed".to_string(),
        diagnostics: Vec::new(),
    })
}

pub fn reconcile_stage(
    request: &ReconciliationStageRequest,
) -> Result<ReconciliationStageResponse, ToolError> {
    let parsed = parse_request_decimals(request)?;
    if parsed.source_total != parsed.extracted_total {
        return Ok(blocked_response(
            "reconcile",
            "validate:passed|reconcile:blocked",
            vec!["totals_mismatch".to_string()],
            vec![ReconciliationDiagnostic {
                key: "source_extracted_total_mismatch".to_string(),
                message: "source_total and extracted_total must match".to_string(),
            }],
        ));
    }

    Ok(ReconciliationStageResponse {
        stage: "reconcile".to_string(),
        status: "passed".to_string(),
        blocked_reasons: Vec::new(),
        stage_marker: "validate:passed|reconcile:passed".to_string(),
        diagnostics: Vec::new(),
    })
}

pub fn commit_stage(
    request: &ReconciliationStageRequest,
) -> Result<ReconciliationStageResponse, ToolError> {
    let parsed = parse_request_decimals(request)?;
    let reconcile = reconcile_stage(request)?;
    if reconcile.status == "blocked" {
        return Ok(blocked_response(
            "commit",
            "validate:passed|reconcile:blocked|commit:blocked",
            reconcile.blocked_reasons,
            reconcile.diagnostics,
        ));
    }

    if parsed.posting_sum != Decimal::ZERO {
        return Ok(blocked_response(
            "commit",
            "validate:passed|reconcile:passed|commit:blocked",
            vec!["imbalance_postings".to_string()],
            vec![ReconciliationDiagnostic {
                key: "posting_balance_mismatch".to_string(),
                message: "posting amounts must net to 0.00".to_string(),
            }],
        ));
    }

    Ok(ReconciliationStageResponse {
        stage: "commit".to_string(),
        status: "ready".to_string(),
        blocked_reasons: Vec::new(),
        stage_marker: "validate:passed|reconcile:passed|commit:ready".to_string(),
        diagnostics: Vec::new(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedRequest {
    source_total: Decimal,
    extracted_total: Decimal,
    posting_sum: Decimal,
}

fn parse_request_decimals(request: &ReconciliationStageRequest) -> Result<ParsedRequest, ToolError> {
    let source_total = parse_decimal_field("source_total", &request.source_total)?;
    let extracted_total = parse_decimal_field("extracted_total", &request.extracted_total)?;
    let posting_sum = request
        .posting_amounts
        .iter()
        .enumerate()
        .try_fold(Decimal::ZERO, |acc, (index, amount)| {
            parse_decimal_field(&format!("posting_amounts[{index}]"), amount).map(|v| acc + v)
        })?;

    Ok(ParsedRequest {
        source_total,
        extracted_total,
        posting_sum,
    })
}

fn parse_decimal_field(field: &str, value: &str) -> Result<Decimal, ToolError> {
    Decimal::from_str(value).map_err(|_| ToolError::InvalidInput(format!("{field} must be a decimal")))
}

fn blocked_response(
    stage: &str,
    marker: &str,
    mut blocked_reasons: Vec<String>,
    mut diagnostics: Vec<ReconciliationDiagnostic>,
) -> ReconciliationStageResponse {
    blocked_reasons.sort();
    diagnostics.sort_by(|a, b| a.key.cmp(&b.key));
    ReconciliationStageResponse {
        stage: stage.to_string(),
        status: "blocked".to_string(),
        blocked_reasons,
        stage_marker: marker.to_string(),
        diagnostics,
    }
}
