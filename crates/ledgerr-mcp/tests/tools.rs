/// Tests #5a and #5b — MCP tool registry contains calendar and shape tools.
///
/// These tests live in ledgerr-mcp (not ledger-core) so they can import
/// TOOL_REGISTRY directly without a cross-crate dev-dependency.
use ledgerr_mcp::contract::TOOL_REGISTRY;

#[test]
fn test_mcp_list_calendar_events_tool_exists() {
    assert!(
        TOOL_REGISTRY.contains(&"list_calendar_events"),
        "TOOL_REGISTRY must contain 'list_calendar_events'; got: {TOOL_REGISTRY:?}"
    );
}

#[test]
fn test_mcp_get_document_shape_tool_exists() {
    assert!(
        TOOL_REGISTRY.contains(&"get_document_shape"),
        "TOOL_REGISTRY must contain 'get_document_shape'; got: {TOOL_REGISTRY:?}"
    );
}

#[test]
fn test_tool_registry_has_all_published_tools() {
    // Sanity: every PUBLISHED_TOOLS name must also appear in TOOL_REGISTRY.
    use ledgerr_mcp::contract::PUBLISHED_TOOLS;
    for spec in &PUBLISHED_TOOLS {
        assert!(
            TOOL_REGISTRY.contains(&spec.name),
            "TOOL_REGISTRY missing published tool '{}'; update TOOL_REGISTRY in contract.rs",
            spec.name
        );
    }
}
