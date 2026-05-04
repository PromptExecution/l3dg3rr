//! Re-exports from ledgerr-mcp-core for backward compatibility.
//!
//! The `McpProvider` trait, `StdioMcpProvider`, `McpProviderRegistry`, and
//! related types were extracted to `ledgerr-mcp-core` so they can be used by
//! `ledgerr-host`, `ledgerr-tauri`, and other crates without pulling in the
//! full `ledgerr-mcp` dependency tree.  This module re-exports everything
//! so existing import paths (`use ledgerr_mcp::provider::*`) continue to work.

pub use ledgerr_mcp_core::*;
