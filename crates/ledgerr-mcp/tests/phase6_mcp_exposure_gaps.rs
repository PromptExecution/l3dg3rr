#![allow(unexpected_cfgs)]
#![cfg(phase6_gap_tests)]

mod common;

/// Phase 6: MCP Exposure Gaps — Failing Test Suite
///
/// These tests document capabilities that exist in the `TurboLedgerTools` service API
/// but are NOT yet exposed as MCP tools. Every test in this file is expected to FAIL
/// until the corresponding MCP wiring is added, so this suite is opt-in and skipped
/// by default in normal CI/test runs.
///
/// Priority labels follow `docs/mcp-capability-contract.md` Section 6:
///   P0 — Mission-critical; blocks AI-first classification workflows
///   P1 — High value; required for CPA-ready outputs
///   P2 — Valuable; completes the write/curate surface
///
/// Reproduction:
/// `RUSTFLAGS="--cfg phase6_gap_tests" cargo test --test phase6_mcp_exposure_gaps`
/// All failures are tracked in GitHub Issues with labels per AGENTS.md protocol.
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

// ─── MCP stdio test harness (same pattern as mcp_stdio_e2e.rs) ──────────────

struct McpClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpClient {
    fn spawn() -> Self {
        let server_bin = env!("CARGO_BIN_EXE_ledgerr-mcp-server");
        let mut child = Command::new(server_bin)
            .env(
                "LEDGERR_MCP_MANIFEST",
                common::stdio_test_manifest("phase6-gaps"),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn turbo-mcp-server");
        let stdin = child.stdin.take().expect("server stdin");
        let stdout = BufReader::new(child.stdout.take().expect("server stdout"));
        Self {
            child,
            stdin,
            stdout,
            next_id: 1,
        }
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        let payload = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        let line = serde_json::to_string(&payload).expect("serialize");
        writeln!(self.stdin, "{line}").expect("write");
        self.stdin.flush().expect("flush");
        let mut response = String::new();
        self.stdout.read_line(&mut response).expect("read line");
        serde_json::from_str::<Value>(response.trim()).expect("parse json")
    }

    fn initialized(&mut self) {
        self.request(
            "initialize",
            json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": { "name": "phase6-gap-tests", "version": "0.1.0" }
            }),
        );
        let notif = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {},
        });
        let line = serde_json::to_string(&notif).expect("serialize notif");
        writeln!(self.stdin, "{line}").expect("write notif");
        self.stdin.flush().expect("flush notif");
    }

    fn tool_names(&mut self) -> Vec<String> {
        let tools = self.request("tools/list", json!({}));
        tools["result"]["tools"]
            .as_array()
            .expect("tools array")
            .iter()
            .filter_map(|t| t.get("name").and_then(Value::as_str).map(str::to_owned))
            .collect()
    }

    fn call(&mut self, name: &str, arguments: Value) -> Value {
        self.request(
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        )
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ─── Helper: ingest one transaction through MCP so classify/flag tools have data ─

fn ingest_one_tx(client: &mut McpClient, dir: &tempfile::TempDir) -> String {
    let result = client.call(
        "proxy_docling_ingest_pdf",
        json!({
            "pdf_path": "WF--BH-CHK--2023-01--statement.pdf",
            "journal_path": dir.path().join("ledger.beancount").display().to_string(),
            "workbook_path": dir.path().join("tax.xlsx").display().to_string(),
            "raw_context_bytes": [99, 116, 120],
            "extracted_rows": [{
                "account_id": "WF-BH-CHK",
                "date": "2023-06-15",
                "amount": "-42.00",
                "description": "Unknown Vendor",
                "source_ref": dir.path().join("stmt.rkyv").display().to_string()
            }]
        }),
    );
    assert_eq!(
        result["result"]["isError"],
        Value::Bool(false),
        "setup ingest must succeed"
    );
    parse_response_payload(&result["result"])["tx_ids"][0]
        .as_str()
        .expect("tx_id")
        .to_owned()
}

fn write_rhai_rule(dir: &tempfile::TempDir) -> String {
    let path = dir.path().join("classify.rhai");
    std::fs::write(
        &path,
        r#"fn classify(tx) {
    let desc = tx["description"];
    let category = if desc.contains("Coffee") { "Meals" } else { "Uncategorized" };
    let confidence = if category == "Meals" { 0.92 } else { 0.33 };
    #{ category: category, confidence: confidence, review: confidence < 0.80, reason: "phase6" }
}"#,
    )
    .expect("write rhai rule");
    path.display().to_string()
}

// ═══════════════════════════════════════════════════════════════════════════════
// P0: l3dg3rr_classify_ingested
//
// Backend: TurboLedgerTools::classify_ingested / classify_ingested service method
// Status:  SERVICE ONLY — not in tool_catalog(), no dispatch arm in turbo-mcp-server
// Impact:  Agents cannot trigger batch classification without direct code execution.
// Issue:   File as GitHub Issue with label `turbo-mcp`, severity P0.
// ═══════════════════════════════════════════════════════════════════════════════

/// P0-GAP: l3dg3rr_classify_ingested must appear in tools/list.
///
/// Expected failure: tool not present in `mcp_adapter::tool_catalog()`.
/// Fix: add `l3dg3rr_classify_ingested` constant + dispatch arm in
///      `mcp_adapter.rs` and `turbo-mcp-server.rs`.
#[test]
fn p0_classify_ingested_is_advertised_in_tool_catalog() {
    let mut client = McpClient::spawn();
    client.initialized();
    let names = client.tool_names();
    assert!(
        names.iter().any(|n| n == "l3dg3rr_classify_ingested"),
        "P0: `l3dg3rr_classify_ingested` must be listed in tools/list but is absent.\n\
         Present tools: {names:?}"
    );
}

/// P0-GAP: Calling l3dg3rr_classify_ingested must return isError:false.
///
/// Expected failure: tool dispatch falls through to `unknown_tool_result`,
///                   returning isError:true with message "unknown tool: l3dg3rr_classify_ingested".
#[test]
fn p0_classify_ingested_call_returns_success() {
    let mut client = McpClient::spawn();
    client.initialized();
    let dir = tempfile::tempdir().expect("tempdir");
    ingest_one_tx(&mut client, &dir); // pre-populate state
    let rule_file = write_rhai_rule(&dir);

    let result = client.call(
        "l3dg3rr_classify_ingested",
        json!({
            "rule_file": rule_file,
            "review_threshold": "0.80"
        }),
    );
    assert_eq!(
        result["result"]["isError"],
        Value::Bool(false),
        "P0: l3dg3rr_classify_ingested must succeed, got:\n{}",
        serde_json::to_string_pretty(&result).unwrap_or_default()
    );
    let classifications = parse_response_payload(&result["result"])["classifications"]
        .as_array()
        .expect("classifications array must be present");
    assert_eq!(
        classifications.len(),
        1,
        "P0: one ingested tx must produce one classification"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// P0: l3dg3rr_query_flags
//
// Backend: TurboLedgerTools::query_flags
// Status:  SERVICE ONLY — not in tool_catalog(), no dispatch arm
// Impact:  Agents cannot access the human review queue without internal API.
// ═══════════════════════════════════════════════════════════════════════════════

/// P0-GAP: l3dg3rr_query_flags must appear in tools/list.
#[test]
fn p0_query_flags_is_advertised_in_tool_catalog() {
    let mut client = McpClient::spawn();
    client.initialized();
    let names = client.tool_names();
    assert!(
        names.iter().any(|n| n == "l3dg3rr_query_flags"),
        "P0: `l3dg3rr_query_flags` must be listed in tools/list but is absent.\n\
         Present tools: {names:?}"
    );
}

/// P0-GAP: Calling l3dg3rr_query_flags must return isError:false with a flags array.
#[test]
fn p0_query_flags_call_returns_open_review_queue() {
    let mut client = McpClient::spawn();
    client.initialized();
    let dir = tempfile::tempdir().expect("tempdir");
    ingest_one_tx(&mut client, &dir);
    let rule_file = write_rhai_rule(&dir);

    // Classify first to populate flags (low-confidence Uncategorized → review)
    let classify_result = client.call(
        "l3dg3rr_classify_ingested",
        json!({ "rule_file": rule_file, "review_threshold": "0.80" }),
    );
    assert_eq!(
        classify_result["result"]["isError"],
        Value::Bool(false),
        "P0: l3dg3rr_classify_ingested setup must succeed, got:\n{}",
        serde_json::to_string_pretty(&classify_result).unwrap_or_default()
    );

    let result = client.call(
        "l3dg3rr_query_flags",
        json!({ "year": 2023, "status": "open" }),
    );
    assert_eq!(
        result["result"]["isError"],
        Value::Bool(false),
        "P0: l3dg3rr_query_flags must succeed, got:\n{}",
        serde_json::to_string_pretty(&result).unwrap_or_default()
    );
    let flags = parse_response_payload(&result["result"])["flags"]
        .as_array()
        .expect("flags array must be present");
    assert_eq!(
        flags.len(),
        1,
        "P0: one low-confidence tx must be in open review queue"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// P0: l3dg3rr_query_audit_log
//
// Backend: TurboLedgerTools::query_audit_log
// Status:  SERVICE ONLY — not in tool_catalog(), no dispatch arm
// Impact:  Agents cannot retrieve audit trail; breaks CPA explainability chain.
// ═══════════════════════════════════════════════════════════════════════════════

/// P0-GAP: l3dg3rr_query_audit_log must appear in tools/list.
#[test]
fn p0_query_audit_log_is_advertised_in_tool_catalog() {
    let mut client = McpClient::spawn();
    client.initialized();
    let names = client.tool_names();
    assert!(
        names.iter().any(|n| n == "l3dg3rr_query_audit_log"),
        "P0: `l3dg3rr_query_audit_log` must be listed in tools/list but is absent.\n\
         Present tools: {names:?}"
    );
}

/// P0-GAP: Calling l3dg3rr_query_audit_log must return isError:false with an entries array.
#[test]
fn p0_query_audit_log_call_returns_audit_entries() {
    let mut client = McpClient::spawn();
    client.initialized();
    let dir = tempfile::tempdir().expect("tempdir");
    let tx_id = ingest_one_tx(&mut client, &dir);

    // Manually classify to generate an audit entry
    let classify_result = client.call(
        "l3dg3rr_classify_transaction",
        json!({
            "tx_id": tx_id,
            "category": "OfficeSupplies",
            "confidence": "0.88",
            "actor": "test-agent",
            "note": "phase6 audit test"
        }),
    );
    assert_eq!(
        classify_result["result"]["isError"],
        Value::Bool(false),
        "P0: l3dg3rr_classify_transaction setup must succeed, got:\n{}",
        serde_json::to_string_pretty(&classify_result).unwrap_or_default()
    );

    let result = client.call("l3dg3rr_query_audit_log", json!({}));
    assert_eq!(
        result["result"]["isError"],
        Value::Bool(false),
        "P0: l3dg3rr_query_audit_log must succeed, got:\n{}",
        serde_json::to_string_pretty(&result).unwrap_or_default()
    );
    let entries = parse_response_payload(&result["result"])["entries"]
        .as_array()
        .expect("entries array must be present");
    assert!(
        !entries.is_empty(),
        "P0: audit log must contain at least one entry after classification"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// P1: l3dg3rr_classify_transaction
//
// Backend: TurboLedgerTools::classify_transaction
// Status:  SERVICE ONLY — not in tool_catalog(), no dispatch arm
// Impact:  Agents cannot apply explicit per-transaction corrections over MCP.
// ═══════════════════════════════════════════════════════════════════════════════

/// P1-GAP: l3dg3rr_classify_transaction must appear in tools/list.
#[test]
fn p1_classify_transaction_is_advertised_in_tool_catalog() {
    let mut client = McpClient::spawn();
    client.initialized();
    let names = client.tool_names();
    assert!(
        names.iter().any(|n| n == "l3dg3rr_classify_transaction"),
        "P1: `l3dg3rr_classify_transaction` must be listed in tools/list but is absent.\n\
         Present tools: {names:?}"
    );
}

/// P1-GAP: Calling l3dg3rr_classify_transaction must persist the classification
///         and return the updated category + audit entries.
#[test]
fn p1_classify_transaction_call_persists_and_returns_audit() {
    let mut client = McpClient::spawn();
    client.initialized();
    let dir = tempfile::tempdir().expect("tempdir");
    let tx_id = ingest_one_tx(&mut client, &dir);

    let result = client.call(
        "l3dg3rr_classify_transaction",
        json!({
            "tx_id": &tx_id,
            "category": "OfficeSupplies",
            "confidence": "0.93",
            "actor": "test-agent",
            "note": "phase6 manual correction"
        }),
    );
    assert_eq!(
        result["result"]["isError"],
        Value::Bool(false),
        "P1: l3dg3rr_classify_transaction must succeed, got:\n{}",
        serde_json::to_string_pretty(&result).unwrap_or_default()
    );
    assert_eq!(
        parse_response_payload(&result["result"])["category"],
        json!("OfficeSupplies"),
        "P1: category must be persisted"
    );
    let entries = parse_response_payload(&result["result"])["audit_entries"]
        .as_array()
        .expect("audit_entries must be present");
    assert!(
        !entries.is_empty(),
        "P1: at least one audit entry must be produced"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// P1: l3dg3rr_reconcile_excel_classification
//
// Backend: TurboLedgerTools::reconcile_excel_classification
// Status:  SERVICE ONLY — not in tool_catalog(), no dispatch arm
// Impact:  Manual Excel edits cannot be synced back into the audit/event chain.
// ═══════════════════════════════════════════════════════════════════════════════

/// P1-GAP: l3dg3rr_reconcile_excel_classification must appear in tools/list.
#[test]
fn p1_reconcile_excel_classification_is_advertised_in_tool_catalog() {
    let mut client = McpClient::spawn();
    client.initialized();
    let names = client.tool_names();
    assert!(
        names.iter().any(|n| n == "l3dg3rr_reconcile_excel_classification"),
        "P1: `l3dg3rr_reconcile_excel_classification` must be listed in tools/list but is absent.\n\
         Present tools: {names:?}"
    );
}

/// P1-GAP: Calling l3dg3rr_reconcile_excel_classification must return the
///         reconciled classification with audit trail — same contract as classify_transaction
///         but semantically marking the source as a human Excel edit.
#[test]
fn p1_reconcile_excel_classification_call_returns_updated_category() {
    let mut client = McpClient::spawn();
    client.initialized();
    let dir = tempfile::tempdir().expect("tempdir");
    let tx_id = ingest_one_tx(&mut client, &dir);

    let result = client.call(
        "l3dg3rr_reconcile_excel_classification",
        json!({
            "tx_id": &tx_id,
            "category": "Travel",
            "confidence": "0.85",
            "actor": "cpa-excel-sync",
            "note": "CPA override from spreadsheet"
        }),
    );
    assert_eq!(
        result["result"]["isError"],
        Value::Bool(false),
        "P1: l3dg3rr_reconcile_excel_classification must succeed, got:\n{}",
        serde_json::to_string_pretty(&result).unwrap_or_default()
    );
    assert_eq!(
        parse_response_payload(&result["result"])["category"],
        json!("Travel"),
        "P1: Excel-reconciled category must be persisted"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// P1: l3dg3rr_get_schedule_summary
//
// Backend: TurboLedgerTools::get_schedule_summary
// Status:  SERVICE ONLY — not in tool_catalog(), no dispatch arm
// Impact:  Agents cannot retrieve compact tax-schedule materializations over MCP.
// ═══════════════════════════════════════════════════════════════════════════════

/// P1-GAP: l3dg3rr_get_schedule_summary must appear in tools/list.
#[test]
fn p1_get_schedule_summary_is_advertised_in_tool_catalog() {
    let mut client = McpClient::spawn();
    client.initialized();
    let names = client.tool_names();
    assert!(
        names.iter().any(|n| n == "l3dg3rr_get_schedule_summary"),
        "P1: `l3dg3rr_get_schedule_summary` must be listed in tools/list but is absent.\n\
         Present tools: {names:?}"
    );
}

/// P1-GAP: Calling l3dg3rr_get_schedule_summary must return year, schedule, total, lines.
#[test]
fn p1_get_schedule_summary_call_returns_schedule_c_payload() {
    let mut client = McpClient::spawn();
    client.initialized();
    let dir = tempfile::tempdir().expect("tempdir");
    let tx_id = ingest_one_tx(&mut client, &dir);

    let classify_result = client.call(
        "l3dg3rr_classify_transaction",
        json!({
            "tx_id": &tx_id,
            "category": "OfficeSupplies",
            "confidence": "0.90",
            "actor": "test-agent"
        }),
    );
    assert_eq!(
        classify_result["result"]["isError"],
        Value::Bool(false),
        "P1: l3dg3rr_classify_transaction setup must succeed, got:\n{}",
        serde_json::to_string_pretty(&classify_result).unwrap_or_default()
    );

    let result = client.call(
        "l3dg3rr_get_schedule_summary",
        json!({ "year": 2023, "schedule": "ScheduleC" }),
    );
    assert_eq!(
        result["result"]["isError"],
        Value::Bool(false),
        "P1: l3dg3rr_get_schedule_summary must succeed, got:\n{}",
        serde_json::to_string_pretty(&result).unwrap_or_default()
    );
    let json = parse_response_payload(&result["result"]);
    assert_eq!(json["year"], json!(2023), "P1: year must be present");
    assert_eq!(
        json["schedule"],
        json!("ScheduleC"),
        "P1: schedule must be present"
    );
    assert!(json.get("total").is_some(), "P1: total must be present");
    assert!(json["lines"].is_array(), "P1: lines must be an array");
}

// ═══════════════════════════════════════════════════════════════════════════════
// P2: l3dg3rr_export_cpa_workbook
//
// Backend: TurboLedgerTools::export_cpa_workbook
// Status:  SERVICE ONLY — not in tool_catalog(), no dispatch arm
// Impact:  Agents cannot trigger CPA handoff artifact generation over MCP.
// ═══════════════════════════════════════════════════════════════════════════════

/// P2-GAP: l3dg3rr_export_cpa_workbook must appear in tools/list.
#[test]
fn p2_export_cpa_workbook_is_advertised_in_tool_catalog() {
    let mut client = McpClient::spawn();
    client.initialized();
    let names = client.tool_names();
    assert!(
        names.iter().any(|n| n == "l3dg3rr_export_cpa_workbook"),
        "P2: `l3dg3rr_export_cpa_workbook` must be listed in tools/list but is absent.\n\
         Present tools: {names:?}"
    );
}

/// P2-GAP: Calling l3dg3rr_export_cpa_workbook must return sheets_written > 0.
#[test]
fn p2_export_cpa_workbook_call_produces_workbook() {
    let mut client = McpClient::spawn();
    client.initialized();
    let dir = tempfile::tempdir().expect("tempdir");
    let tx_id = ingest_one_tx(&mut client, &dir);
    let classify_result = client.call(
        "l3dg3rr_classify_transaction",
        json!({
            "tx_id": &tx_id,
            "category": "OfficeSupplies",
            "confidence": "0.90",
            "actor": "test-agent"
        }),
    );
    assert_eq!(
        classify_result["result"]["isError"],
        Value::Bool(false),
        "P2: l3dg3rr_classify_transaction setup must succeed, got:\n{}",
        serde_json::to_string_pretty(&classify_result).unwrap_or_default()
    );

    let workbook_path = dir.path().join("cpa-output.xlsx");
    let result = client.call(
        "l3dg3rr_export_cpa_workbook",
        json!({ "workbook_path": workbook_path.display().to_string() }),
    );
    assert_eq!(
        result["result"]["isError"],
        Value::Bool(false),
        "P2: l3dg3rr_export_cpa_workbook must succeed, got:\n{}",
        serde_json::to_string_pretty(&result).unwrap_or_default()
    );
    let sheets_written = parse_response_payload(&result["result"])["sheets_written"]
        .as_u64()
        .expect("sheets_written must be a number");
    assert!(sheets_written > 0, "P2: at least one sheet must be written");
    assert!(
        workbook_path.exists(),
        "P2: workbook file must exist on disk"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// P2: l3dg3rr_ontology_upsert_entities / l3dg3rr_ontology_upsert_edges
//
// Backend: TurboLedgerService::ontology_upsert_entities_tool /
//          TurboLedgerService::ontology_upsert_edges_tool
// Status:  SERVICE ONLY — not in tool_catalog(), no dispatch arm
// Impact:  Agents can only read the ontology; cannot curate it over MCP.
// ═══════════════════════════════════════════════════════════════════════════════

/// P2-GAP: l3dg3rr_ontology_upsert_entities must appear in tools/list.
#[test]
fn p2_ontology_upsert_entities_is_advertised_in_tool_catalog() {
    let mut client = McpClient::spawn();
    client.initialized();
    let names = client.tool_names();
    assert!(
        names
            .iter()
            .any(|n| n == "l3dg3rr_ontology_upsert_entities"),
        "P2: `l3dg3rr_ontology_upsert_entities` must be listed in tools/list but is absent.\n\
         Present tools: {names:?}"
    );
}

/// P2-GAP: l3dg3rr_ontology_upsert_edges must appear in tools/list.
#[test]
fn p2_ontology_upsert_edges_is_advertised_in_tool_catalog() {
    let mut client = McpClient::spawn();
    client.initialized();
    let names = client.tool_names();
    assert!(
        names.iter().any(|n| n == "l3dg3rr_ontology_upsert_edges"),
        "P2: `l3dg3rr_ontology_upsert_edges` must be listed in tools/list but is absent.\n\
         Present tools: {names:?}"
    );
}

/// P2-GAP: Calling l3dg3rr_ontology_upsert_entities must persist entities and return count.
#[test]
fn p2_ontology_upsert_entities_call_persists_entity() {
    let mut client = McpClient::spawn();
    client.initialized();
    let dir = tempfile::tempdir().expect("tempdir");
    let ontology_path = dir.path().join("ontology.json");

    let result = client.call(
        "l3dg3rr_ontology_upsert_entities",
        json!({
            "ontology_path": ontology_path.display().to_string(),
            "entities": [{
                "id": "WF-BH-CHK",
                "kind": "Account",
                "label": "Wells Fargo Checking",
                "properties": {}
            }]
        }),
    );
    assert_eq!(
        result["result"]["isError"],
        Value::Bool(false),
        "P2: l3dg3rr_ontology_upsert_entities must succeed, got:\n{}",
        serde_json::to_string_pretty(&result).unwrap_or_default()
    );
    let upserted = parse_response_payload(&result["result"])["upserted"]
        .as_u64()
        .expect("upserted count must be present");
    assert_eq!(upserted, 1, "P2: one entity must be reported as upserted");
}

/// P2-GAP: Calling l3dg3rr_ontology_upsert_edges must persist edges and return count.
#[test]
fn p2_ontology_upsert_edges_call_persists_edge() {
    let mut client = McpClient::spawn();
    client.initialized();
    let dir = tempfile::tempdir().expect("tempdir");
    let ontology_path = dir.path().join("ontology.json");

    // Seed entities first. This test intentionally depends on upsert_entities (P2)
    // working — edge upsert requires entities to exist. Assert success so failures
    // point to the broken step rather than the edge assertion downstream.
    let entities_result = client.call(
        "l3dg3rr_ontology_upsert_entities",
        json!({
            "ontology_path": ontology_path.display().to_string(),
            "entities": [
                { "id": "WF-BH-CHK", "kind": "Account", "label": "Checking", "properties": {} },
                { "id": "TXN-001",    "kind": "Transaction", "label": "Coffee",  "properties": {} }
            ]
        }),
    );
    assert_eq!(
        entities_result["result"]["isError"],
        Value::Bool(false),
        "P2: l3dg3rr_ontology_upsert_entities setup must succeed (2 entities), got:\n{}",
        serde_json::to_string_pretty(&entities_result).unwrap_or_default()
    );

    let result = client.call(
        "l3dg3rr_ontology_upsert_edges",
        json!({
            "ontology_path": ontology_path.display().to_string(),
            "edges": [{
                "from_id": "WF-BH-CHK",
                "to_id": "TXN-001",
                "relation": "HasTransaction"
            }]
        }),
    );
    assert_eq!(
        result["result"]["isError"],
        Value::Bool(false),
        "P2: l3dg3rr_ontology_upsert_edges must succeed, got:\n{}",
        serde_json::to_string_pretty(&result).unwrap_or_default()
    );
    let upserted = parse_response_payload(&result["result"])["upserted"]
        .as_u64()
        .expect("upserted edge count must be present");
    assert_eq!(upserted, 1, "P2: one edge must be reported as upserted");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Invariant: Confidence values must survive Decimal round-trip without precision loss
//
// The workspace invariant requires `rust_decimal::Decimal` for ALL domain values.
// `RunRhaiRuleResponse.confidence` and `ClassifiedTxResponse.confidence` are
// currently typed as `f64`, which can silently lose precision for values
// like 0.92 (not representable exactly in IEEE 754 binary floating point).
//
// These tests verify that confidence values round-trip through Decimal cleanly.
// ═══════════════════════════════════════════════════════════════════════════════

/// INVARIANT: confidence returned by classify_ingested must not lose decimal precision.
///
/// Expected failure mode: f64 representation of 0.92 → 0.9199999... when
/// compared against Decimal("0.92") via exact string matching.
#[test]
fn invariant_classify_ingested_confidence_is_decimal_exact() {
    use ledger_core::ingest::TransactionInput;
    use ledgerr_mcp::{
        ClassifyIngestedRequest, IngestPdfRequest, TurboLedgerService, TurboLedgerTools,
    };

    let workbook_path = common::unique_workbook_path("phase6-classify-ingested");
    let svc =
        TurboLedgerService::from_manifest_str(&common::manifest_for_workbook(&workbook_path, 2023))
            .expect("manifest");

    let dir = tempfile::tempdir().expect("tempdir");
    svc.ingest_pdf(IngestPdfRequest {
        pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
        journal_path: dir.path().join("ledger.beancount"),
        workbook_path: dir.path().join("tax.xlsx"),
        ontology_path: None,
        raw_context_bytes: Some(b"ctx".to_vec()),
        extracted_rows: vec![TransactionInput {
            account_id: "WF-BH-CHK".to_string(),
            date: "2023-01-15".to_string(),
            amount: "-42.00".to_string(),
            description: "Coffee Beans".to_string(),
            source_ref: dir.path().join("stmt.rkyv").display().to_string(),
        }],
    })
    .expect("ingest");

    let rule_path = dir.path().join("classify.rhai");
    std::fs::write(
        &rule_path,
        r#"fn classify(tx) {
    #{ category: "Meals", confidence: 0.92, review: false, reason: "decimal-test" }
}"#,
    )
    .expect("write rule");

    let classified = svc
        .classify_ingested(ClassifyIngestedRequest {
            rule_file: rule_path,
            review_threshold: 0.80,
        })
        .expect("classify");

    assert_eq!(classified.classifications.len(), 1);
    let conf = classified.classifications[0].confidence;

    // Rust's default f64 Display (Ryu) prints "0.92" even for the imprecise binary
    // float, so `format!("{conf}")` would silently pass when confidence is f64.
    // Instead, convert via Decimal::from_f64 and compare against the exact parsed
    // value — Decimal::from_f64(0.92_f64) != Decimal::from_str("0.92") because the
    // IEEE 754 representation of 0.92 is 0.91999999999999993..., exposing the violation.
    use rust_decimal::prelude::*;
    let conf_as_decimal = Decimal::from_f64(conf).expect("confidence must be a finite number");
    let expected = Decimal::from_str("0.92").expect("parse exact decimal");
    assert_eq!(
        conf_as_decimal, expected,
        "INVARIANT: confidence must be representable as exact Decimal; \
         Decimal::from_f64({conf}) = {conf_as_decimal} != {expected}. \
         Fix: change RunRhaiRuleResponse.confidence to rust_decimal::Decimal."
    );
}

/// INVARIANT: confidence returned by query_flags must not lose decimal precision.
#[test]
fn invariant_query_flags_confidence_is_decimal_exact() {
    use ledger_core::ingest::TransactionInput;
    use ledgerr_mcp::{
        ClassifyIngestedRequest, FlagStatusRequest, IngestPdfRequest, QueryFlagsRequest,
        TurboLedgerService, TurboLedgerTools,
    };

    let workbook_path = common::unique_workbook_path("phase6-query-flags");
    let svc =
        TurboLedgerService::from_manifest_str(&common::manifest_for_workbook(&workbook_path, 2023))
            .expect("manifest");

    let dir = tempfile::tempdir().expect("tempdir");
    svc.ingest_pdf(IngestPdfRequest {
        pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
        journal_path: dir.path().join("ledger.beancount"),
        workbook_path: dir.path().join("tax.xlsx"),
        ontology_path: None,
        raw_context_bytes: Some(b"ctx".to_vec()),
        extracted_rows: vec![TransactionInput {
            account_id: "WF-BH-CHK".to_string(),
            date: "2023-01-15".to_string(),
            amount: "-99.00".to_string(),
            description: "Unknown Vendor".to_string(),
            source_ref: dir.path().join("stmt.rkyv").display().to_string(),
        }],
    })
    .expect("ingest");

    let rule_path = dir.path().join("classify.rhai");
    std::fs::write(
        &rule_path,
        r#"fn classify(tx) {
    #{ category: "Uncategorized", confidence: 0.33, review: true, reason: "decimal-test" }
}"#,
    )
    .expect("write rule");

    svc.classify_ingested(ClassifyIngestedRequest {
        rule_file: rule_path,
        review_threshold: 0.80,
    })
    .expect("classify");

    let flags = svc
        .query_flags(QueryFlagsRequest {
            year: 2023,
            status: FlagStatusRequest::Open,
        })
        .expect("query flags");

    assert_eq!(flags.flags.len(), 1);
    let conf = flags.flags[0].confidence;

    // Same reasoning as the classify_ingested invariant: Ryu Display hides f64
    // imprecision for 0.33. Use Decimal conversion to reliably detect the violation.
    use rust_decimal::prelude::*;
    let conf_as_decimal = Decimal::from_f64(conf).expect("confidence must be a finite number");
    let expected = Decimal::from_str("0.33").expect("parse exact decimal");
    assert_eq!(
        conf_as_decimal, expected,
        "INVARIANT: flag confidence must be exact Decimal; \
         Decimal::from_f64({conf}) = {conf_as_decimal} != {expected}. \
         Fix: change FlagRecordResponse.confidence to rust_decimal::Decimal."
    );
}
mod common;
