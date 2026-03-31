use ledger_core::ingest::TransactionInput;

// DOC-01 (D-01, D-03): MCP transport boundary must expose proxy/passthrough tools.
#[test]
fn doc_01_mcp_boundary_tool_catalog_exposes_passthrough_proxy_surface() {
    let tools = turbo_mcp::mcp_adapter::tool_catalog();

    assert!(tools.contains(&"proxy_rustledger_ingest_statement_rows".to_string()));
    assert!(tools.contains(&"proxy_docling_ingest_pdf".to_string()));
    assert!(tools.contains(&"l3dg3rr_list_accounts".to_string()));
    assert!(tools.contains(&"l3dg3rr_get_raw_context".to_string()));
    assert!(tools.contains(&"l3dg3rr_get_pipeline_status".to_string()));
    assert!(tools.contains(&"tools/list".to_string()));
    assert!(tools.contains(&"tools/call".to_string()));
}

// DOC-02 (D-02, D-04): Canonical rows + provenance fields must be deterministic.
#[test]
fn doc_02_normalized_rows_include_canonical_and_provenance_fields() {
    let rows = vec![TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-15".to_string(),
        amount: "-42.11".to_string(),
        description: "Coffee Shop".to_string(),
        source_ref: "2023-taxes/WF--BH-CHK--2023-01--statement.rkyv".to_string(),
    }];

    let normalized = turbo_mcp::mcp_adapter::normalize_rows_with_provenance(
        "rustledger",
        "ingest_statement_rows",
        Some("1.0.0"),
        Some("call-001"),
        rows,
    );

    assert_eq!(normalized.len(), 1);
    let entry = &normalized[0];

    assert!(entry.get("account").is_some());
    assert!(entry.get("date").is_some());
    assert!(entry.get("amount").is_some());
    assert!(entry.get("description").is_some());
    assert!(entry.get("currency").is_some());
    assert!(entry.get("source_ref").is_some());
    assert!(entry.get("provider").is_some());
    assert!(entry.get("backend_tool").is_some());
    assert!(entry.get("backend_version").is_some());
    assert!(entry.get("backend_call_id").is_some());
}

// DOC-02 (D-04): Stable enum-like status + blockers + next_hint contract.
#[test]
fn doc_02_pipeline_status_shape_is_deterministic_and_concise() {
    let status = turbo_mcp::mcp_adapter::get_pipeline_status(
        true,
        true,
        false,
        vec!["docling_unreachable".to_string()],
    );

    assert_eq!(status.status, "blocked");
    assert_eq!(status.blockers, vec!["docling_unreachable".to_string()]);
    assert_eq!(status.next_hint, "resolve_blockers_then_retry");
}

// DOC-01 (D-03): Rustledger proxy surface must remain explicitly advertised in catalog.
#[test]
fn doc_01_rustledger_proxy_tool_name_is_exact_and_callable_target() {
    let tools = turbo_mcp::mcp_adapter::tool_catalog();
    assert!(tools
        .iter()
        .any(|name| name == "proxy_rustledger_ingest_statement_rows"));
}
