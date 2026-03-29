use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

const HSM_TRANSITION_TOOL: &str = "l3dg3rr_hsm_transition";
const HSM_STATUS_TOOL: &str = "l3dg3rr_hsm_status";
const HSM_RESUME_TOOL: &str = "l3dg3rr_hsm_resume";

struct McpStdioClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpStdioClient {
    fn spawn() -> Self {
        let server_bin = env!("CARGO_BIN_EXE_turbo-mcp-server");
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
            "clientInfo": { "name": "hsm-mcp-e2e", "version": "0.1.0" }
        }),
    );
    assert!(initialize.get("result").is_some(), "initialize must succeed");
    client.send_notification_initialized();
}

#[test]
fn hsm_03_tools_list_includes_transition_status_and_resume_tools() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let tools = client.request("tools/list", json!({}));
    let tool_names = tools["result"]["tools"]
        .as_array()
        .expect("tools list")
        .iter()
        .filter_map(|entry| entry.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert!(tool_names.contains(&HSM_TRANSITION_TOOL));
    assert!(tool_names.contains(&HSM_STATUS_TOOL));
    assert!(tool_names.contains(&HSM_RESUME_TOOL));
}

#[test]
fn hsm_03_invalid_transition_and_resume_return_deterministic_blocked_payloads() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let transition = client.request(
        "tools/call",
        json!({
            "name": HSM_TRANSITION_TOOL,
            "arguments": {
                "target_state": "reconcile",
                "target_substate": "ready"
            }
        }),
    );
    assert_eq!(transition["result"]["isError"], Value::Bool(true));
    let blocked_transition = &transition["result"]["content"][0]["json"];
    assert_eq!(blocked_transition["error_type"], json!("HsmTransitionBlocked"));
    assert_eq!(blocked_transition["guard_reason"], json!("invalid_transition"));
    assert_eq!(blocked_transition["transition_evidence"], json!([
        "from=ingest.pending",
        "to=reconcile.ready",
        "allowed=normalize.ready"
    ]));

    let resume = client.request(
        "tools/call",
        json!({
            "name": HSM_RESUME_TOOL,
            "arguments": {
                "state_marker": "validate:ready:advanced"
            }
        }),
    );
    assert_eq!(resume["result"]["isError"], Value::Bool(true));
    let blocked_resume = &resume["result"]["content"][0]["json"];
    assert_eq!(blocked_resume["error_type"], json!("HsmResumeBlocked"));
    assert_eq!(blocked_resume["blockers"], json!(["checkpoint_unknown"]));
}

#[test]
fn hsm_03_status_and_resume_payload_include_small_model_hint_fields() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let status = client.request(
        "tools/call",
        json!({
            "name": HSM_STATUS_TOOL,
            "arguments": {}
        }),
    );
    assert_eq!(status["result"]["isError"], Value::Bool(false));
    let status_payload = &status["result"]["content"][0]["json"];
    assert_eq!(status_payload["display_state"], json!("ingest.pending"));
    assert_eq!(status_payload["next_hint"], json!("advance_to_normalize"));
    assert_eq!(status_payload["resume_hint"], json!("resume_from_ingest.pending"));
    assert_eq!(status_payload["blockers"], json!([]));

    let resume = client.request(
        "tools/call",
        json!({
            "name": HSM_RESUME_TOOL,
            "arguments": {
                "state_marker": "ingest:pending:advanced"
            }
        }),
    );
    assert_eq!(resume["result"]["isError"], Value::Bool(false));
    let resume_payload = &resume["result"]["content"][0]["json"];
    assert_eq!(resume_payload["resume_from"], json!("ingest:pending:advanced"));
    assert_eq!(resume_payload["resume_hint"], json!("advance_to_normalize"));
    assert_eq!(resume_payload["blockers"], json!([]));
}
