use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

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
            "id": self.next_id,
            "method": "notifications/initialized",
            "params": {},
        });
        self.next_id += 1;
        let _ = self.send_and_read(payload);
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
            "clientInfo": { "name": "mcp-stdio-e2e", "version": "0.1.0" }
        }),
    );
    assert!(initialize.get("result").is_some(), "initialize must succeed");
    client.send_notification_initialized();
}

fn build_ingest_arguments(base_dir: &std::path::Path) -> Value {
    json!({
        "pdf_path": "WF--BH-CHK--2023-01--statement.pdf",
        "journal_path": base_dir.join("ledger.beancount").display().to_string(),
        "workbook_path": base_dir.join("tax-ledger.xlsx").display().to_string(),
        "raw_context_bytes": [99, 116, 120],
        "extracted_rows": [
            {
                "account_id": "WF-BH-CHK",
                "date": "2023-01-15",
                "amount": "-42.11",
                "description": "Coffee Shop",
                "source_ref": base_dir
                    .join("WF--BH-CHK--2023-01--statement.rkyv")
                    .display()
                    .to_string()
            }
        ]
    })
}

// DOC-01: ingest path must be executable through MCP tools/call only.
#[test]
fn doc_01_mcp_only_ingest_via_tools_call() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let tools = client.request("tools/list", json!({}));
    let tool_names = tools["result"]["tools"]
        .as_array()
        .expect("tools list")
        .iter()
        .filter_map(|entry| entry.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(tool_names.contains(&"proxy_docling_ingest_pdf"));

    let tempdir = tempfile::tempdir().expect("tempdir");
    let call = client.request(
        "tools/call",
        json!({
            "name": "proxy_docling_ingest_pdf",
            "arguments": build_ingest_arguments(tempdir.path())
        }),
    );

    assert_eq!(call["result"]["isError"], Value::Bool(false));
    assert_eq!(call["result"]["content"][0]["json"]["inserted_count"], json!(1));
    assert!(call["result"]["content"][0]["json"]["tx_ids"]
        .as_array()
        .expect("tx_ids array")
        .len()
        == 1);
}

// DOC-02: canonical + provenance mapping must be deterministic in MCP payloads.
#[test]
fn doc_02_canonical_mapping_and_provenance_fields_over_transport() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let tempdir = tempfile::tempdir().expect("tempdir");
    let call = client.request(
        "tools/call",
        json!({
            "name": "proxy_docling_ingest_pdf",
            "arguments": build_ingest_arguments(tempdir.path())
        }),
    );

    let canonical = &call["result"]["content"][0]["json"]["canonical_rows"][0];
    assert!(canonical.get("account").is_some());
    assert!(canonical.get("date").is_some());
    assert!(canonical.get("amount").is_some());
    assert!(canonical.get("description").is_some());
    assert!(canonical.get("currency").is_some());
    assert!(canonical.get("source_ref").is_some());
    assert!(canonical.get("provider").is_some());
    assert!(canonical.get("backend_tool").is_some());
    assert!(canonical.get("backend_version").is_some());
    assert!(canonical.get("backend_call_id").is_some());
}

// DOC-03: replaying identical source through MCP remains idempotent with stable tx IDs.
#[test]
fn doc_03_replay_idempotent_with_stable_tx_ids_over_mcp() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);
    let tempdir = tempfile::tempdir().expect("tempdir");

    let first = client.request(
        "tools/call",
        json!({
            "name": "proxy_docling_ingest_pdf",
            "arguments": build_ingest_arguments(tempdir.path())
        }),
    );
    let second = client.request(
        "tools/call",
        json!({
            "name": "proxy_docling_ingest_pdf",
            "arguments": build_ingest_arguments(tempdir.path())
        }),
    );

    assert_eq!(first["result"]["content"][0]["json"]["inserted_count"], json!(1));
    assert_eq!(second["result"]["content"][0]["json"]["inserted_count"], json!(0));

    let first_ids = first["result"]["content"][0]["json"]["tx_ids"]
        .as_array()
        .expect("first tx ids");
    let second_ids = second["result"]["content"][0]["json"]["tx_ids"]
        .as_array()
        .expect("second tx ids");
    assert_eq!(first_ids, second_ids);
}
