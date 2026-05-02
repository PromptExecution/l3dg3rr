//! MCP provider core — reusable MCP provider trait and infrastructure.
//!
//! This crate provides the `McpProvider` trait, `StdioMcpProvider` (subprocess
//! stdio transport over JSON-RPC 2.0), and `McpProviderRegistry` for
//! discovering, routing, and dispatching tools across multiple providers.
//!
//! It has no dependency on `ledgerr-mcp` or `TurboLedgerService`, so it can
//! be used by `ledgerr-host`, `ledgerr-tauri`, or any other crate that needs
//! to talk to external MCP servers.

mod provider;
pub use provider::*;
