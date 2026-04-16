use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

fn parse_response_payload(response: &Value) -> Value {
    let text = response["content"][0]["text"].as_str().unwrap_or("null");
    serde_json::from_str(text).unwrap_or(Value::Null)
}

const PLUGIN_INFO_TOOL: &str = "l3dg3rr_plugin_info";

struct McpStdioClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpStdioClient {
    fn spawn() -> Self {
        let server_bin = env!("CARGO_BIN_EXE_ledgerr-mcp-server");
        let mut child = Command::new(server_bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn ledgerr-mcp-server");
        let stdin = child.stdin.take().expect("server stdin");
        let stdout = BufReader::new(child.stdout.take().expect("server stdout"));
        Self { child, stdin, stdout, next_id: 1 }
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        let payload = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        self.send_and_read(payload)
    }

    fn send_notification_initialized(&mut self) {
        let payload = json!({ "jsonrpc": "2.0", "method": "notifications/initialized", "params": {} });
        let line = serde_json::to_string(&payload).expect("serialize");
        writeln!(self.stdin, "{line}").expect("write");
        self.stdin.flush().expect("flush");
    }

    fn send_and_read(&mut self, payload: Value) -> Value {
        let line = serde_json::to_string(&payload).expect("serialize");
        writeln!(self.stdin, "{line}").expect("write");
        self.stdin.flush().expect("flush");
        let mut response = String::new();
        self.stdout.read_line(&mut response).expect("read");
        serde_json::from_str::<Value>(response.trim()).expect("parse json")
    }
}

impl Drop for McpStdioClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn initialize_client(client: &mut McpStdioClient) {
    let resp = client.request(
        "initialize",
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "plugin-info-mcp-e2e", "version": "0.1.0" }
        }),
    );
    assert!(resp.get("result").is_some(), "initialize must succeed");
    client.send_notification_initialized();
}

// ── tools/list ────────────────────────────────────────────────────────────────

#[test]
fn pi_01_tools_list_advertises_plugin_info() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let tools = client.request("tools/list", json!({}));
    let names = tools["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect::<Vec<_>>();

    assert!(
        names.contains(&PLUGIN_INFO_TOOL),
        "tools/list must include {PLUGIN_INFO_TOOL}; got: {names:?}"
    );
}

#[test]
fn pi_01_plugin_info_schema_has_subcommand_enum() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let tools = client.request("tools/list", json!({}));
    let schema = tools["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .find(|t| t["name"] == PLUGIN_INFO_TOOL)
        .expect("plugin_info in tools/list")["inputSchema"]
        .clone();

    let enum_values = schema["properties"]["subcommand"]["enum"]
        .as_array()
        .expect("subcommand enum");
    let variants: Vec<&str> = enum_values.iter().filter_map(Value::as_str).collect();
    assert!(variants.contains(&"check"));
    assert!(variants.contains(&"upgrade"));
    assert!(variants.contains(&"cleanup"));
}

// ── subcommand: check (default) ───────────────────────────────────────────────

#[test]
fn pi_02_check_returns_version_and_host_metadata() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let resp = client.request(
        "tools/call",
        json!({ "name": PLUGIN_INFO_TOOL, "arguments": {} }),
    );
    assert_eq!(resp["result"]["isError"], Value::Bool(false));

    let p = parse_response_payload(&resp["result"]);
    assert!(p["current_version"].is_string(), "current_version missing");
    assert!(p["latest_version"].is_string(), "latest_version missing");
    assert!(p["update_available"].is_boolean(), "update_available missing");
    assert!(p["log_path"].is_string(), "log_path missing");
    assert!(p["host"].is_object(), "host metadata missing");
    assert!(p["host"]["os"].is_string());
    assert!(p["host"]["arch"].is_string());
    assert!(p["host"]["pid"].is_number());
}

#[test]
fn pi_02_explicit_check_subcommand_is_identical_to_default() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let default_resp = client.request(
        "tools/call",
        json!({ "name": PLUGIN_INFO_TOOL, "arguments": {} }),
    );
    let explicit_resp = client.request(
        "tools/call",
        json!({ "name": PLUGIN_INFO_TOOL, "arguments": { "subcommand": "check" } }),
    );

    let default_p = parse_response_payload(&default_resp["result"]);
    let explicit_p = parse_response_payload(&explicit_resp["result"]);

    // Both must report the same embedded version.
    assert_eq!(default_p["current_version"], explicit_p["current_version"]);
    assert_eq!(default_p["update_available"], explicit_p["update_available"]);
}

#[test]
fn pi_02_current_version_is_non_empty_semver_like() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let resp = client.request(
        "tools/call",
        json!({ "name": PLUGIN_INFO_TOOL, "arguments": {} }),
    );
    let p = parse_response_payload(&resp["result"]);
    let version = p["current_version"].as_str().expect("current_version string");

    // Must have at least one dot — x.y or x.y.z shape.
    assert!(version.contains('.'), "version should be semver-like, got: {version}");
}

// ── subcommand: cleanup ───────────────────────────────────────────────────────

#[test]
fn pi_03_cleanup_returns_removed_array_and_count() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let resp = client.request(
        "tools/call",
        json!({ "name": PLUGIN_INFO_TOOL, "arguments": { "subcommand": "cleanup" } }),
    );
    assert_eq!(resp["result"]["isError"], Value::Bool(false));

    let p = parse_response_payload(&resp["result"]);
    assert!(p["removed"].is_array(), "removed must be array");
    assert!(p["count"].is_number(), "count must be number");
    assert!(p["log_path"].is_string(), "log_path must be present");
    // In the test environment there are no .old.exe files next to the test binary.
    assert_eq!(p["count"], json!(0));
}

// ── subcommand: upgrade ───────────────────────────────────────────────────────

#[test]
fn pi_04_upgrade_returns_not_supported_on_non_windows_without_feature() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let resp = client.request(
        "tools/call",
        json!({ "name": PLUGIN_INFO_TOOL, "arguments": { "subcommand": "upgrade" } }),
    );
    assert_eq!(resp["result"]["isError"], Value::Bool(false));

    let p = parse_response_payload(&resp["result"]);

    // On Linux CI (no self-update feature) must report not_supported.
    // On Windows+self-update it reports already_current or proceeds — both are valid statuses.
    let status = p["status"].as_str().expect("status field");
    assert!(
        ["not_supported", "already_current", "upgraded", "error"].contains(&status),
        "unexpected upgrade status: {status}"
    );

    #[cfg(not(all(target_os = "windows", feature = "self-update")))]
    assert_eq!(status, "not_supported");
}

// ── response envelope ─────────────────────────────────────────────────────────

#[test]
fn pi_05_all_subcommands_return_text_content_type() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    for subcommand in ["check", "cleanup", "upgrade"] {
        let resp = client.request(
            "tools/call",
            json!({ "name": PLUGIN_INFO_TOOL, "arguments": { "subcommand": subcommand } }),
        );
        let content_type = resp["result"]["content"][0]["type"].as_str().unwrap_or("");
        assert_eq!(
            content_type, "text",
            "subcommand '{subcommand}' returned content type '{content_type}', expected 'text'"
        );
    }
}

#[test]
fn pi_05_unknown_subcommand_falls_through_to_check() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let resp = client.request(
        "tools/call",
        json!({ "name": PLUGIN_INFO_TOOL, "arguments": { "subcommand": "nonsense" } }),
    );
    assert_eq!(resp["result"]["isError"], Value::Bool(false));

    let p = parse_response_payload(&resp["result"]);
    // Falls through to check — must have current_version.
    assert!(p["current_version"].is_string());
}
