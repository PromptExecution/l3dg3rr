use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};
use turbo_mcp::{
    OntologyEdgeInput, OntologyEntityInput, OntologyEntityKind, OntologyStore,
};

const ONTOLOGY_QUERY_TOOL: &str = "l3dg3rr_ontology_query_path";
const ONTOLOGY_EXPORT_TOOL: &str = "l3dg3rr_ontology_export_snapshot";

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
            "clientInfo": { "name": "ontology-mcp-e2e", "version": "0.1.0" }
        }),
    );
    assert!(initialize.get("result").is_some(), "initialize must succeed");
    client.send_notification_initialized();
}

fn seed_ontology(path: &std::path::Path) -> (String, String, String, String) {
    let mut store = OntologyStore::default();

    let entities = store
        .upsert_entities(vec![
            OntologyEntityInput {
                kind: OntologyEntityKind::Document,
                attrs: {
                    let mut attrs = BTreeMap::new();
                    attrs.insert("source_ref".to_string(), "wf-statement.pdf".to_string());
                    attrs
                },
            },
            OntologyEntityInput {
                kind: OntologyEntityKind::Transaction,
                attrs: {
                    let mut attrs = BTreeMap::new();
                    attrs.insert("tx_id".to_string(), "tx-001".to_string());
                    attrs
                },
            },
            OntologyEntityInput {
                kind: OntologyEntityKind::TaxCategory,
                attrs: {
                    let mut attrs = BTreeMap::new();
                    attrs.insert("category".to_string(), "OfficeSupplies".to_string());
                    attrs
                },
            },
            OntologyEntityInput {
                kind: OntologyEntityKind::EvidenceReference,
                attrs: {
                    let mut attrs = BTreeMap::new();
                    attrs.insert("rkyv_ref".to_string(), "wf-ctx.rkyv".to_string());
                    attrs
                },
            },
        ])
        .expect("seed entities");

    let doc_id = entities.entity_ids[0].clone();
    let tx_id = entities.entity_ids[1].clone();
    let tax_id = entities.entity_ids[2].clone();
    let evidence_id = entities.entity_ids[3].clone();

    store
        .upsert_edges(vec![
            OntologyEdgeInput {
                from: tx_id.clone(),
                to: tax_id.clone(),
                relation: "links_tax_category".to_string(),
                provenance: BTreeMap::new(),
            },
            OntologyEdgeInput {
                from: doc_id.clone(),
                to: tx_id.clone(),
                relation: "documents_transaction".to_string(),
                provenance: BTreeMap::new(),
            },
            OntologyEdgeInput {
                from: tx_id.clone(),
                to: evidence_id.clone(),
                relation: "links_evidence".to_string(),
                provenance: BTreeMap::new(),
            },
        ])
        .expect("seed edges");

    store.persist(path).expect("persist seed ontology");

    (doc_id, tx_id, evidence_id, tax_id)
}

// ONTO-03 (D-03): tools/list advertises ontology query/export transport surfaces.
#[test]
fn onto_03_tools_list_advertises_ontology_query_and_export_tools() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let tools = client.request("tools/list", json!({}));
    let tool_names = tools["result"]["tools"]
        .as_array()
        .expect("tools list")
        .iter()
        .filter_map(|entry| entry.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert!(tool_names.contains(&ONTOLOGY_QUERY_TOOL));
    assert!(tool_names.contains(&ONTOLOGY_EXPORT_TOOL));
}

// ONTO-03 (D-03): ontology query and export return deterministic concise payloads over transport.
#[test]
fn onto_03_tools_call_query_and_export_snapshot_payloads_are_deterministic() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let tempdir = tempfile::tempdir().expect("tempdir");
    let ontology_path = tempdir.path().join("ontology.json");
    let (doc_id, tx_id, evidence_id, tax_id) = seed_ontology(&ontology_path);

    let query = client.request(
        "tools/call",
        json!({
            "name": ONTOLOGY_QUERY_TOOL,
            "arguments": {
                "ontology_path": ontology_path.display().to_string(),
                "from_entity_id": doc_id,
                "max_depth": 4
            }
        }),
    );

    assert_eq!(query["result"]["isError"], Value::Bool(false));

    let query_json = &query["result"]["content"][0]["json"];
    let node_ids = query_json["nodes"]
        .as_array()
        .expect("nodes array")
        .iter()
        .map(|node| node["id"].as_str().expect("node id").to_string())
        .collect::<Vec<_>>();
    assert_eq!(node_ids, vec![doc_id, tx_id, evidence_id, tax_id]);

    let export = client.request(
        "tools/call",
        json!({
            "name": ONTOLOGY_EXPORT_TOOL,
            "arguments": {
                "ontology_path": ontology_path.display().to_string()
            }
        }),
    );

    assert_eq!(export["result"]["isError"], Value::Bool(false));

    let export_json = &export["result"]["content"][0]["json"];
    let entity_kinds = export_json["entities"]
        .as_array()
        .expect("entities array")
        .iter()
        .map(|entity| entity["kind"].as_str().expect("kind").to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        entity_kinds,
        vec![
            "document".to_string(),
            "transaction".to_string(),
            "tax_category".to_string(),
            "evidence_reference".to_string(),
        ]
    );

    let edge_relations = export_json["edges"]
        .as_array()
        .expect("edges array")
        .iter()
        .map(|edge| edge["relation"].as_str().expect("relation").to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        edge_relations,
        vec![
            "documents_transaction".to_string(),
            "links_evidence".to_string(),
            "links_tax_category".to_string(),
        ]
    );
}

// ONTO-03 (D-03): repeated ontology export for unchanged inputs remains byte-for-byte stable.
#[test]
fn onto_03_export_snapshot_stable_json_serialization_over_transport() {
    let mut client = McpStdioClient::spawn();
    initialize_client(&mut client);

    let tempdir = tempfile::tempdir().expect("tempdir");
    let ontology_path = tempdir.path().join("ontology.json");
    let _ = seed_ontology(&ontology_path);

    let first = client.request(
        "tools/call",
        json!({
            "name": ONTOLOGY_EXPORT_TOOL,
            "arguments": {
                "ontology_path": ontology_path.display().to_string()
            }
        }),
    );
    let second = client.request(
        "tools/call",
        json!({
            "name": ONTOLOGY_EXPORT_TOOL,
            "arguments": {
                "ontology_path": ontology_path.display().to_string()
            }
        }),
    );

    assert_eq!(first["result"]["isError"], Value::Bool(false));
    assert_eq!(second["result"]["isError"], Value::Bool(false));

    let first_payload = serde_json::to_string(&first["result"]["content"][0]["json"])
        .expect("serialize first payload");
    let second_payload = serde_json::to_string(&second["result"]["content"][0]["json"])
        .expect("serialize second payload");

    assert_eq!(first_payload, second_payload);
}
