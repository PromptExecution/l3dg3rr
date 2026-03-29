use std::io::{self, BufRead, Write};

use serde_json::{json, Value};
use turbo_mcp::mcp_adapter;

fn main() {
    // Serve a minimal stdio MCP transport boundary for initialize/tools/list/tools/call.
    // Stdout is reserved for protocol payloads only.
    serve(io::stdin().lock(), io::stdout());
}

fn serve<R: BufRead, W: Write>(reader: R, mut writer: W) {
    for line in reader.lines() {
        let Ok(raw) = line else { continue };
        let Ok(request) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        if let Some(response) = handle_request(request) {
            if let Ok(serialized) = serde_json::to_string(&response) {
                let _ = writeln!(writer, "{serialized}");
                let _ = writer.flush();
            }
        }
    }
}

fn handle_request(request: Value) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");

    match method {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2025-11-25",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "turbo-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })),
        "tools/list" => {
            let tools: Vec<Value> = mcp_adapter::tool_catalog()
                .into_iter()
                .map(|name| json!({ "name": name }))
                .collect();
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "tools": tools }
            }))
        }
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or(Value::Null);
            let tool_name = params.get("name").and_then(Value::as_str).unwrap_or("");
            if tool_name == "l3dg3rr_get_pipeline_status" {
                let status = mcp_adapter::get_pipeline_status(true, true, true, Vec::new());
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{
                            "type": "json",
                            "json": status
                        }],
                        "isError": false
                    }
                }))
            } else {
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{
                            "type": "json",
                            "json": {
                                "isError": true,
                                "error_type": "InvalidInput",
                                "message": format!("unknown tool: {tool_name}")
                            }
                        }],
                        "isError": true
                    }
                }))
            }
        }
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("method not found: {method}")
            }
        })),
    }
}
