use std::sync::Arc;

use ledgerr_mcp::provider::McpProvider;

// ============================================================================
// McpProvider compile-and-construct integration test.
//
// Requires the "b00t" feature on ledgerr-mcp (enabled in dev-dependencies
// in Cargo.toml).  Tests the McpProviderRegistry, MockProvider, and
// register_default_providers graceful degradation.
// ============================================================================

#[test]
fn smoke_registry_construct_empty() {
    let registry = ledgerr_mcp::provider::McpProviderRegistry::new();
    let results = registry.initialize_all();
    assert!(results.is_empty(), "empty registry should return no results");
}

#[test]
fn smoke_registry_with_mock_providers() {
    let mut registry = ledgerr_mcp::provider::McpProviderRegistry::new();
    registry.register(Arc::new(
        ledgerr_mcp::provider::mock::MockProvider::new("smoke-b00t", "b00t_status"),
    ));
    registry.register(Arc::new(
        ledgerr_mcp::provider::mock::MockProvider::new("smoke-just", "just_recipes"),
    ));

    let results = registry.initialize_all();
    assert_eq!(results.len(), 2);
    for (name, result) in &results {
        assert!(result.is_ok(), "mock provider {name} should initialize ok: {result:?}");
    }

    let descriptors = registry.all_tool_descriptors();
    assert_eq!(descriptors.len(), 2);
    let tool_names: Vec<_> = descriptors.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"b00t_status"));
    assert!(tool_names.contains(&"just_recipes"));
}

#[test]
fn smoke_registry_call_tool_by_provider_name() {
    let mut registry = ledgerr_mcp::provider::McpProviderRegistry::new();
    registry.register(Arc::new(
        ledgerr_mcp::provider::mock::MockProvider::new("calc", "add"),
    ));

    let result = registry.call_tool("calc", "add", serde_json::json!({}));
    assert!(result.is_ok(), "call by provider name: {result:?}");
}

#[test]
fn smoke_registry_call_tool_by_tool_name_fallback() {
    let mut registry = ledgerr_mcp::provider::McpProviderRegistry::new();
    registry.register(Arc::new(
        ledgerr_mcp::provider::mock::MockProvider::new("calc", "add"),
    ));

    let result = registry.call_tool("", "add", serde_json::json!({}));
    assert!(result.is_ok(), "call by tool name fallback: {result:?}");
}

#[test]
fn smoke_registry_unknown_tool_returns_error() {
    let mut registry = ledgerr_mcp::provider::McpProviderRegistry::new();
    registry.register(Arc::new(
        ledgerr_mcp::provider::mock::MockProvider::new("calc", "add"),
    ));

    let result = registry.call_tool("", "nonexistent", serde_json::json!({}));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no provider found"));
}

#[test]
fn smoke_registry_find_provider() {
    let mut registry = ledgerr_mcp::provider::McpProviderRegistry::new();
    registry.register(Arc::new(
        ledgerr_mcp::provider::mock::MockProvider::new("b00t", "b00t_status"),
    ));

    let found = registry.find_provider("b00t_status");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name(), "b00t");

    let not_found = registry.find_provider("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn smoke_mock_provider_call_count() {
    use std::sync::atomic::Ordering;

    let provider = ledgerr_mcp::provider::mock::MockProvider::new("counter", "inc");
    let provider = Arc::new(provider);

    let _ = provider.call_tool("inc", serde_json::json!({}));
    let _ = provider.call_tool("inc", serde_json::json!({}));
    let _ = provider.call_tool("inc", serde_json::json!({}));

    assert_eq!(
        provider.call_count.load(Ordering::SeqCst),
        3,
        "mock should count 3 calls"
    );
}

#[test]
fn smoke_register_default_providers_graceful_missing_binaries() {
    let mut registry = ledgerr_mcp::provider::McpProviderRegistry::new();
    ledgerr_mcp::providers::definitions::register_default_providers(
        &mut registry,
        Some(std::path::PathBuf::from("/nonexistent/b00t-home")),
        Some(std::path::PathBuf::from("/nonexistent/project")),
    );

    let results = registry.initialize_all();
    assert!(results.len() <= 3, "at most 3 provider entries");
    for (name, result) in &results {
        match result {
            Ok(info) => {
                assert!(!info.name.is_empty(), "provider {name} returned empty name");
            }
            Err(e) => {
                let msg = e.to_string();
                assert!(!msg.is_empty(), "error should have a message for {name}");
            }
        }
    }
}
