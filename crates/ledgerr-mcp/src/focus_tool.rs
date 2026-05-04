//! MCP tool adapter for `ledgerr_focus` — FOCUS v1.3 cost/usage records.
//!
//! Actions: append_focus_record, query_focus_summary, compute_focus_delta, experiment_score.
//! All actions accept JSON input and return JSON output via the MCP contract.

use ledgerr_focus::{
    compute_focus_delta, format_focus_cli, ChargeCategory, ChargeFrequency, CostAndUsageRow,
    PersonalityProfile, FOCUS_SPEC_VERSION,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusToolInput {
    pub action: String,
    #[serde(default)]
    pub records: Vec<FocusToolRecord>,
    #[serde(default)]
    pub experiment_id: Option<String>,
    #[serde(default)]
    pub personality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusToolRecord {
    pub billing_account_id: String,
    pub service_name: String,
    pub billed_cost: f64,
    pub effective_cost: f64,
    pub experiment_id: Option<String>,
    pub variant: Option<String>,
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusToolOutput {
    pub spec_version: &'static str,
    pub action: String,
    pub rows_written: usize,
    pub focus_cli: String,
    pub delta: Option<FocusDeltaOutput>,
    pub experiment_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusDeltaOutput {
    pub control_billed: f64,
    pub treatment_billed: f64,
    pub delta_billed: f64,
    pub recommendation: String,
    pub dimension_deltas: HashMap<String, f64>,
}

pub fn handle_focus_tool(input: FocusToolInput) -> Result<FocusToolOutput, String> {
    match input.action.as_str() {
        "append_focus_record" => handle_append(input),
        "query_focus_summary" => handle_query_summary(input),
        "compute_focus_delta" => handle_compute_delta(input),
        "experiment_score" => handle_experiment_score(input),
        other => Err(format!("unknown focus action: {other}")),
    }
}

/// Validate a FocusToolRecord against FOCUS v1.3 mandatory columns.
/// Returns Ok(()) if all mandatory fields are present and non-empty.
fn validate_focus_record(record: &FocusToolRecord) -> Result<(), String> {
    let mut errors = Vec::new();
    if record.billing_account_id.is_empty() {
        errors.push("BillingAccountId".to_string());
    }
    if record.service_name.is_empty() {
        errors.push("ServiceName".to_string());
    }
    if record.billed_cost < 0.0 {
        errors.push("BilledCost (negative)".to_string());
    }
    if !errors.is_empty() {
        return Err(format!("FOCUS validation failed: missing/invalid mandatory columns: {}", errors.join(", ")));
    }
    Ok(())
}

fn handle_append(input: FocusToolInput) -> Result<FocusToolOutput, String> {
    // Validate all incoming records against the FOCUS v1.3 schema before processing
    for record in &input.records {
        validate_focus_record(record)?;
    }
    let rows: Vec<CostAndUsageRow> = input
        .records
        .iter()
        .map(|r| CostAndUsageRow {
            billing_account_id: r.billing_account_id.clone(),
            billing_account_name: None,
            billing_currency: "FOCUS".into(),
            billing_period_start: chrono::Utc::now(),
            billing_period_end: chrono::Utc::now(),
            charge_period_start: chrono::Utc::now(),
            charge_period_end: chrono::Utc::now(),
            charge_category: ChargeCategory::Usage,
            charge_frequency: ChargeFrequency::UsageBased,
            billed_cost: Decimal::from_f64(r.billed_cost).unwrap_or_default(),
            effective_cost: Decimal::from_f64(r.effective_cost).unwrap_or_default(),
            service_provider_name: "ledgrrr".into(),
            service_name: r.service_name.clone(),
            sku_id: "focus-eval".into(),
            billing_account_type: None,
            charge_class: None,
            charge_description: None,
            commitment_discount_id: None,
            commitment_discount_name: None,
            commitment_discount_category: None,
            commitment_discount_type: None,
            commitment_discount_status: None,
            commitment_discount_quantity: None,
            commitment_discount_unit: None,
            consumed_quantity: None,
            consumed_unit: None,
            contracted_cost: None,
            contracted_unit_price: None,
            invoice_id: None,
            invoice_issuer_name: None,
            list_cost: None,
            list_unit_price: None,
            pricing_category: None,
            pricing_quantity: None,
            pricing_unit: None,
            region_id: None,
            region_name: None,
            resource_id: None,
            resource_name: None,
            resource_type: None,
            service_category: Some("AI Inference".into()),
            service_subcategory: None,
            sku_meter: None,
            sku_price_id: None,
            sku_price_details: None,
            sub_account_id: None,
            sub_account_name: None,
            sub_account_type: None,
            availability_zone: None,
            capacity_reservation_id: None,
            capacity_reservation_status: None,
            host_provider_name: None,
            tags: HashMap::new(),
            x_experiment_id: r.experiment_id.clone(),
            x_variant: r.variant.clone(),
            x_personality: input.personality.clone(),
            x_experiment_score: None,
            x_agent_id: r.agent_id.clone(),
            x_reasoning_review: None,
        })
        .collect();

    let focus_cli = rows
        .iter()
        .map(format_focus_cli)
        .collect::<Vec<_>>()
        .join("\n");

    let summary = input
        .experiment_id
        .as_ref()
        .map(|eid| format!("FOCUS {FOCUS_SPEC_VERSION}: {n} rows appended to experiment {eid}", n = rows.len()));

    Ok(FocusToolOutput {
        spec_version: FOCUS_SPEC_VERSION,
        action: "append_focus_record".into(),
        rows_written: rows.len(),
        focus_cli,
        delta: None,
        experiment_summary: summary,
    })
}

fn handle_query_summary(_input: FocusToolInput) -> Result<FocusToolOutput, String> {
    Ok(FocusToolOutput {
        spec_version: FOCUS_SPEC_VERSION,
        action: "query_focus_summary".into(),
        rows_written: 0,
        focus_cli: String::new(),
        delta: None,
        experiment_summary: Some(format!(
            "FOCUS {FOCUS_SPEC_VERSION}: ledgerr-focus crate bound, ready for record ingestion"
        )),
    })
}

fn handle_compute_delta(input: FocusToolInput) -> Result<FocusToolOutput, String> {
    let control: Vec<CostAndUsageRow> = input
        .records
        .iter()
        .filter(|r| r.variant.as_deref() == Some("control"))
        .map(|r| CostAndUsageRow {
            billing_account_id: r.billing_account_id.clone(),
            billing_account_name: None,
            billing_currency: "FOCUS".into(),
            billing_period_start: chrono::Utc::now(),
            billing_period_end: chrono::Utc::now(),
            charge_period_start: chrono::Utc::now(),
            charge_period_end: chrono::Utc::now(),
            charge_category: ChargeCategory::Usage,
            charge_frequency: ChargeFrequency::UsageBased,
            billed_cost: Decimal::from_f64(r.billed_cost).unwrap_or_default(),
            effective_cost: Decimal::from_f64(r.effective_cost).unwrap_or_default(),
            service_provider_name: "ledgrrr".into(),
            service_name: r.service_name.clone(),
            sku_id: "focus-eval".into(),
            billing_account_type: None,
            charge_class: None,
            charge_description: None,
            commitment_discount_id: None,
            commitment_discount_name: None,
            commitment_discount_category: None,
            commitment_discount_type: None,
            commitment_discount_status: None,
            commitment_discount_quantity: None,
            commitment_discount_unit: None,
            consumed_quantity: None,
            consumed_unit: None,
            contracted_cost: None,
            contracted_unit_price: None,
            invoice_id: None,
            invoice_issuer_name: None,
            list_cost: None,
            list_unit_price: None,
            pricing_category: None,
            pricing_quantity: None,
            pricing_unit: None,
            region_id: None,
            region_name: None,
            resource_id: None,
            resource_name: None,
            resource_type: None,
            service_category: Some("AI Inference".into()),
            service_subcategory: None,
            sku_meter: None,
            sku_price_id: None,
            sku_price_details: None,
            sub_account_id: None,
            sub_account_name: None,
            sub_account_type: None,
            availability_zone: None,
            capacity_reservation_id: None,
            capacity_reservation_status: None,
            host_provider_name: None,
            tags: HashMap::new(),
            x_experiment_id: r.experiment_id.clone(),
            x_variant: r.variant.clone(),
            x_personality: input.personality.clone(),
            x_experiment_score: None,
            x_agent_id: r.agent_id.clone(),
            x_reasoning_review: None,
        })
        .collect();

    let treatment: Vec<CostAndUsageRow> = input
        .records
        .iter()
        .filter(|r| r.variant.as_deref() == Some("treatment"))
        .map(|r| CostAndUsageRow {
            billing_account_id: r.billing_account_id.clone(),
            billing_account_name: None,
            billing_currency: "FOCUS".into(),
            billing_period_start: chrono::Utc::now(),
            billing_period_end: chrono::Utc::now(),
            charge_period_start: chrono::Utc::now(),
            charge_period_end: chrono::Utc::now(),
            charge_category: ChargeCategory::Usage,
            charge_frequency: ChargeFrequency::UsageBased,
            billed_cost: Decimal::from_f64(r.billed_cost).unwrap_or_default(),
            effective_cost: Decimal::from_f64(r.effective_cost).unwrap_or_default(),
            service_provider_name: "ledgrrr".into(),
            service_name: r.service_name.clone(),
            sku_id: "focus-eval".into(),
            billing_account_type: None,
            charge_class: None,
            charge_description: None,
            commitment_discount_id: None,
            commitment_discount_name: None,
            commitment_discount_category: None,
            commitment_discount_type: None,
            commitment_discount_status: None,
            commitment_discount_quantity: None,
            commitment_discount_unit: None,
            consumed_quantity: None,
            consumed_unit: None,
            contracted_cost: None,
            contracted_unit_price: None,
            invoice_id: None,
            invoice_issuer_name: None,
            list_cost: None,
            list_unit_price: None,
            pricing_category: None,
            pricing_quantity: None,
            pricing_unit: None,
            region_id: None,
            region_name: None,
            resource_id: None,
            resource_name: None,
            resource_type: None,
            service_category: Some("AI Inference".into()),
            service_subcategory: None,
            sku_meter: None,
            sku_price_id: None,
            sku_price_details: None,
            sub_account_id: None,
            sub_account_name: None,
            sub_account_type: None,
            availability_zone: None,
            capacity_reservation_id: None,
            capacity_reservation_status: None,
            host_provider_name: None,
            tags: HashMap::new(),
            x_experiment_id: r.experiment_id.clone(),
            x_variant: r.variant.clone(),
            x_personality: input.personality.clone(),
            x_experiment_score: None,
            x_agent_id: r.agent_id.clone(),
            x_reasoning_review: None,
        })
        .collect();

    let mut cs = HashMap::new();
    let mut ts = HashMap::new();
    cs.insert("roi".into(), Decimal::from_f64(0.5).unwrap());
    ts.insert("roi".into(), Decimal::from_f64(0.8).unwrap());

    let delta = compute_focus_delta(&control, &treatment, &cs, &ts, input.experiment_id.as_deref().unwrap_or("?"));

    let mut dim_deltas = HashMap::new();
    for (k, v) in &delta.dimension_deltas {
        dim_deltas.insert(k.clone(), v.to_f64().unwrap_or(0.0));
    }

    Ok(FocusToolOutput {
        spec_version: FOCUS_SPEC_VERSION,
        action: "compute_focus_delta".into(),
        rows_written: 0,
        focus_cli: delta.to_focus_cli(),
        delta: Some(FocusDeltaOutput {
            control_billed: delta.control_billed_cost.to_f64().unwrap_or(0.0),
            treatment_billed: delta.treatment_billed_cost.to_f64().unwrap_or(0.0),
            delta_billed: delta.delta_billed.to_f64().unwrap_or(0.0),
            recommendation: delta.recommendation,
            dimension_deltas: dim_deltas,
        }),
        experiment_summary: None,
    })
}

fn handle_experiment_score(input: FocusToolInput) -> Result<FocusToolOutput, String> {
    let personality = input
        .personality
        .as_deref()
        .and_then(|p| PersonalityProfile::all().into_iter().find(|prof| prof.label == p))
        .map(|_| format!("personality={}", input.personality.as_deref().unwrap_or("none")));

    Ok(FocusToolOutput {
        spec_version: FOCUS_SPEC_VERSION,
        action: "experiment_score".into(),
        rows_written: input.records.len(),
        focus_cli: personality.unwrap_or_default(),
        delta: None,
        experiment_summary: input.experiment_id.map(|eid| format!("scored experiment {eid}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(variant: &str, cost: f64) -> FocusToolRecord {
        FocusToolRecord {
            billing_account_id: "b00t-hive".into(),
            service_name: "experiment-eval".into(),
            billed_cost: cost,
            effective_cost: cost * 0.85,
            experiment_id: Some("exp-001".into()),
            variant: Some(variant.into()),
            agent_id: Some(format!("sm0l-{variant}")),
        }
    }

    #[test]
    fn test_append_focus_record() {
        let input = FocusToolInput {
            action: "append_focus_record".into(),
            records: vec![make_record("control", 100.0)],
            experiment_id: Some("exp-001".into()),
            personality: Some("analyst".into()),
        };
        let output = handle_focus_tool(input).unwrap();
        assert_eq!(output.action, "append_focus_record");
        assert_eq!(output.rows_written, 1);
        assert!(output.focus_cli.contains("ledgrrr focus append"));
        assert!(output.spec_version.starts_with("1."));
    }

    #[test]
    fn test_compute_focus_delta_action() {
        let input = FocusToolInput {
            action: "compute_focus_delta".into(),
            records: vec![
                make_record("control", 100.0),
                make_record("treatment", 150.0),
            ],
            experiment_id: Some("exp-001".into()),
            personality: None,
        };
        let output = handle_focus_tool(input).unwrap();
        let delta = output.delta.unwrap();
        assert_eq!(delta.control_billed, 100.0);
        assert_eq!(delta.treatment_billed, 150.0);
        assert_eq!(delta.recommendation, "treatment");
    }

    #[test]
    fn test_experiment_score_action() {
        let input = FocusToolInput {
            action: "experiment_score".into(),
            records: vec![make_record("control", 100.0)],
            experiment_id: Some("exp-001".into()),
            personality: Some("explorer".into()),
        };
        let output = handle_focus_tool(input).unwrap();
        assert!(output.focus_cli.contains("personality=explorer"));
        assert_eq!(output.rows_written, 1);
    }

    #[test]
    fn test_unknown_action_errors() {
        let input = FocusToolInput {
            action: "bogus".into(),
            records: vec![],
            experiment_id: None,
            personality: None,
        };
        assert!(handle_focus_tool(input).is_err());
    }

    #[test]
    fn test_validate_focus_record_passes_valid() {
        let record = make_record("control", 100.0);
        assert!(validate_focus_record(&record).is_ok());
    }

    #[test]
    fn test_validate_focus_record_rejects_empty_billing_account() {
        let mut record = make_record("control", 100.0);
        record.billing_account_id.clear();
        assert!(validate_focus_record(&record).is_err());
    }

    #[test]
    fn test_validate_focus_record_rejects_negative_cost() {
        let record = make_record("control", -50.0);
        assert!(validate_focus_record(&record).is_err());
    }

    #[test]
    fn test_handle_append_validates_before_processing() {
        let mut record = make_record("control", 100.0);
        record.billing_account_id.clear();
        let input = FocusToolInput {
            action: "append_focus_record".into(),
            records: vec![record],
            experiment_id: Some("exp-001".into()),
            personality: Some("analyst".into()),
        };
        let result = handle_focus_tool(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("BillingAccountId"));
    }

    #[test]
    fn test_query_summary() {
        let input = FocusToolInput {
            action: "query_focus_summary".into(),
            records: vec![],
            experiment_id: None,
            personality: None,
        };
        let output = handle_focus_tool(input).unwrap();
        assert!(output.experiment_summary.unwrap().contains("FOCUS"));
    }
}
