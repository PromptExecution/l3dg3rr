use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

const RECON_VALIDATE_TOOL: &str = "l3dg3rr_validate_reconciliation";
const RECON_RECONCILE_TOOL: &str = "l3dg3rr_reconcile_postings";
const RECON_COMMIT_TOOL: &str = "l3dg3rr_commit_guarded";

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
        let payload = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        self.send_and_read(payload)
    }

    fn send_notification_initialized(&mut self) {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {},
        });
        let line = serde_json::to_string(&payload).expect("serialize notification");
        writeln!(self.stdin, "{line}").expect("write notification");
        self.stdin.flush().expect("flush notification");
    }

    fn send_and_read(&mut self, payload: Value) -> Value {
        let line = serde_json::to_string(&payload).expect("serialize request");
        writeln!(self.stdin, "{line}").expect("write request");
        self.stdin.flush().expect("flush request");

        let mut response = String::new();
        self.stdout
            .read_line(&mut response)
            .expect("read response line");
        serde_json::from_str::<Value>(response.trim()).expect("parse response json")
    }
}

impl Drop for McpStdioClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn initialize_client(client: &mut McpStdioClient) {
    let initialize = client.request(
        "initialize",
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "reconciliation-mcp-e2e", "version": "0.1.0" }
        }),
    );
    assert!(
        initialize.get("result").is_some(),
        "initialize must succeed"
    );
    client.send_notification_initialized();
}

fn balanced_request() -> Value {
    json!({
        "source_total": "100.00",
        "extracted_total": "100.00",
        "posting_amounts": ["-100.00", "100.00"]
    })
}

fn imbalanced_request() -> Value {
    json!({
        "source_total": "100.00",
        "extracted_total": "100.00",
        "posting_amounts": ["-100.00", "90.00"]
    })
}

#[test]
fn recon_03_tools_list_includes_reconciliation_stage_tools() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let tools = client.request("tools/list", json!({}));
    let tool_names = tools["result"]["tools"]
        .as_array()
        .expect("tools list")
        .iter()
        .filter_map(|entry| entry.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert!(tool_names.contains(&RECON_VALIDATE_TOOL));
    assert!(tool_names.contains(&RECON_RECONCILE_TOOL));
    assert!(tool_names.contains(&RECON_COMMIT_TOOL));
}

#[test]
fn recon_03_failing_commit_returns_deterministic_blocking_diagnostics() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let commit = client.request(
        "tools/call",
        json!({
            "name": RECON_COMMIT_TOOL,
            "arguments": imbalanced_request()
        }),
    );

    assert_eq!(commit["result"]["isError"], Value::Bool(true));
    let payload = &commit["result"]["content"][0]["json"];
    assert_eq!(payload["error_type"], json!("ReconciliationBlocked"));
    assert_eq!(
        payload["message"],
        json!("commit blocked by reconciliation guardrails")
    );
    assert_eq!(payload["stage"], json!("commit"));
    assert_eq!(
        payload["stage_marker"],
        json!("validate:passed|reconcile:passed|commit:blocked")
    );
    assert_eq!(payload["blocked_reasons"], json!(["imbalance_postings"]));
}

#[test]
fn recon_03_validate_and_reconcile_then_commit_returns_explicit_ready_payload() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let validate = client.request(
        "tools/call",
        json!({
            "name": RECON_VALIDATE_TOOL,
            "arguments": balanced_request()
        }),
    );
    assert_eq!(validate["result"]["isError"], Value::Bool(false));
    assert_eq!(
        validate["result"]["content"][0]["json"]["stage"],
        json!("validate")
    );
    assert_eq!(
        validate["result"]["content"][0]["json"]["status"],
        json!("passed")
    );

    let reconcile = client.request(
        "tools/call",
        json!({
            "name": RECON_RECONCILE_TOOL,
            "arguments": balanced_request()
        }),
    );
    assert_eq!(reconcile["result"]["isError"], Value::Bool(false));
    assert_eq!(
        reconcile["result"]["content"][0]["json"]["stage"],
        json!("reconcile")
    );
    assert_eq!(
        reconcile["result"]["content"][0]["json"]["status"],
        json!("passed")
    );

    let commit = client.request(
        "tools/call",
        json!({
            "name": RECON_COMMIT_TOOL,
            "arguments": balanced_request()
        }),
    );
    assert_eq!(commit["result"]["isError"], Value::Bool(false));

    let payload = &commit["result"]["content"][0]["json"];
    assert_eq!(payload["stage"], json!("commit"));
    assert_eq!(payload["status"], json!("ready"));
    assert_eq!(
        payload["stage_marker"],
        json!("validate:passed|reconcile:passed|commit:ready")
    );
    assert_eq!(payload["blocked_reasons"], json!([]));
}
