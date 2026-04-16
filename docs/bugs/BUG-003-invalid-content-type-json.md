# BUG-003: All tools return non-spec content type `"json"` — P0

**Severity**: P0  
**Scope**: ALL tools (27 occurrences in `mcp_adapter.rs`)  
**Detected via**: Live MCP tool calls through Cowork MCP bridge  

## Repro

Any tool call via MCP client (e.g. `l3dg3rr_list_accounts`, `l3dg3rr_get_pipeline_status`, etc.) returns:

```json
{
  "content": [{"type": "json", "json": {...}}],
  "isError": false
}
```

MCP client rejects with Zod union validation error on `content[0]`:
```
invalid_union — path: ["content", 0]
  type "json" not in [text, image, audio, resource_link, resource]
```

## Root Cause

`mcp_adapter.rs` constructs all tool results with `"type": "json"` content blocks:
```rust
json!({
    "content": [{"type": "json", "json": payload}],
    "isError": false
})
```

MCP 2025-11-25 spec defines only these content types for `CallToolResult`:
- `text` — `{type: "text", text: string}`
- `image` — `{type: "image", data: string, mimeType: string}`
- `audio` — `{type: "audio", data: string, mimeType: string}`
- `resource_link` — `{type: "resource_link", uri: string, name: string, ...}`
- `resource` — embedded resource object

`"json"` is NOT a valid content type in the spec.

## Fix

Replace every content block from:
```rust
{"type": "json", "json": payload}
```
to:
```rust
{"type": "text", "text": serde_json::to_string(&payload).unwrap_or_default()}
```

Or use a helper like:
```rust
fn json_content(payload: Value) -> Value {
    json!({"type": "text", "text": payload.to_string()})
}
```

There are **27 occurrences** across `mcp_adapter.rs` — all must be updated.

## Impact

**Complete protocol failure** — no tool is callable through any spec-compliant MCP client. The server appeared functional when tested via raw JSON-RPC in the shell (which doesn't validate content type), but fails at the Cowork/Claude MCP bridge layer which validates responses against the spec schema.

## Files

- `crates/ledgerr-mcp/src/mcp_adapter.rs` — all `json!({...content: [{type: json...}]...})` calls
