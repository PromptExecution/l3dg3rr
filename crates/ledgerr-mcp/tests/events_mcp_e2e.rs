use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

fn parse_response_payload(response: &Value) -> Value {
    let text = response["content"][0]["text"].as_str().unwrap_or("null");
    serde_json::from_str(text).unwrap_or(Value::Null)
}

const EVENT_REPLAY_TOOL: &str = "l3dg3rr_event_replay";
const EVENT_HISTORY_TOOL: &str = "l3dg3rr_event_history";

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
            "clientInfo": { "name": "events-mcp-e2e", "version": "0.1.0" }
        }),
    );
    assert!(
        initialize.get("result").is_some(),
        "initialize must succeed"
    );
    client.send_notification_initialized();
}

fn ingest_one_row(
    client: &mut McpStdioClient,
    date: &str,
    description: &str,
    source_ref: &str,
) -> String {
    let ingest = client.request(
        "tools/call",
        json!({
                "name": "proxy_rustledger_ingest_statement_rows",
                "arguments": {
                "journal_path": "/tmp/tax-ledger-journal.beancount",
                "workbook_path": "/tmp/tax-ledger.xlsx",
                "rows": [{
                    "account": "WF-BH-CHK",
                    "date": date,
                    "amount": "-42.11",
                    "description": description,
                    "source_ref": source_ref
                }]
            }
        }),
    );
    assert_eq!(ingest["result"]["isError"], Value::Bool(false));
    let p = parse_response_payload(&ingest["result"]);
    p["tx_ids"][0].as_str().unwrap_or_default().to_string()
}

#[test]
fn evt_03_tools_list_advertises_audit_tool() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let tools = client.request("tools/list", json!({}));
    let tool_names = tools["result"]["tools"]
        .as_array()
        .expect("tools list")
        .iter()
        .filter_map(|entry| entry.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert!(tool_names.contains(&"ledgerr_audit"));
}

#[test]
fn evt_03_event_history_filtering_by_tx_document_and_time_is_deterministic() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let tx_id = ingest_one_row(&mut client, "2023-01-15", "Coffee Shop", "source/a.rkyv");
    let _other_tx = ingest_one_row(&mut client, "2023-02-15", "Groceries", "source/b.rkyv");

    let history = client.request(
        "tools/call",
        json!({
            "name": EVENT_HISTORY_TOOL,
            "arguments": {
                "tx_id": tx_id,
                "document_ref": "source/a.rkyv",
                "time_start": "2023-01-01",
                "time_end": "2023-01-31"
            }
        }),
    );
    assert_eq!(history["result"]["isError"], Value::Bool(false));

    let payload = parse_response_payload(&history["result"]);
    assert_eq!(payload["filter"]["document_ref"], json!("source/a.rkyv"));
    let events = payload["events"].as_array().expect("events");
    assert!(!events.is_empty());
    let mut sequences = events
        .iter()
        .filter_map(|event| event.get("sequence").and_then(Value::as_u64))
        .collect::<Vec<_>>();
    let sorted = {
        sequences.sort();
        sequences.clone()
    };
    assert_eq!(sequences, sorted);
}

#[test]
fn evt_03_invalid_filter_range_returns_deterministic_blocked_envelope() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let response = client.request(
        "tools/call",
        json!({
            "name": EVENT_HISTORY_TOOL,
            "arguments": {
                "time_start": "2023-02-01",
                "time_end": "2023-01-01"
            }
        }),
    );
    assert_eq!(response["result"]["isError"], Value::Bool(true));
    let payload = parse_response_payload(&response["result"]);
    assert_eq!(payload["error_type"], json!("EventHistoryBlocked"));
    assert_eq!(payload["reason"], json!("time_range_invalid"));
}
