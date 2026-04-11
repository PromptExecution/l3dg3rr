use std::collections::BTreeMap;

use ledgerr_mcp::{
    OntologyEdgeInput, OntologyEntityInput, OntologyEntityKind, OntologyQueryPathRequest,
    OntologyUpsertEdgesRequest, OntologyUpsertEntitiesRequest, TurboLedgerService,
};
use tempfile::tempdir;

fn service() -> TurboLedgerService {
    let manifest = r#"
[session]
workbook_path = "tax-ledger.xlsx"
active_year = 2023
"#;

    TurboLedgerService::from_manifest_str(manifest).expect("manifest should parse")
}

// ONTO-01 (D-02, D-03, D-04): valid ontology entities and edges persist with stable content-hash IDs.
#[test]
fn onto_01_persistence_integrity_persists_entities_and_edges_with_stable_ids() {
    let service = service();
    let tmp = tempdir().expect("tempdir");
    let ontology_path = tmp.path().join("ontology.json");

    let mut doc_attrs = BTreeMap::new();
    doc_attrs.insert(
        "source_ref".to_string(),
        "2023/WF--BH-CHK--2023-01--statement.pdf".to_string(),
    );

    let mut tx_attrs = BTreeMap::new();
    tx_attrs.insert("tx_id".to_string(), "tx-001".to_string());

    let entities = service
        .ontology_upsert_entities(OntologyUpsertEntitiesRequest {
            ontology_path: ontology_path.clone(),
            entities: vec![
                OntologyEntityInput {
                    kind: OntologyEntityKind::Document,
                    attrs: doc_attrs,
                },
                OntologyEntityInput {
                    kind: OntologyEntityKind::Transaction,
                    attrs: tx_attrs,
                },
            ],
        })
        .expect("entities should upsert");

    assert_eq!(entities.inserted_count, 2);
    assert_eq!(entities.entity_ids.len(), 2);

    let edges = service
        .ontology_upsert_edges(OntologyUpsertEdgesRequest {
            ontology_path: ontology_path.clone(),
            edges: vec![OntologyEdgeInput {
                from: entities.entity_ids[0].clone(),
                to: entities.entity_ids[1].clone(),
                relation: "documents_transaction".to_string(),
                provenance: BTreeMap::new(),
            }],
        })
        .expect("edge should upsert");

    assert_eq!(edges.inserted_count, 1);
    assert_eq!(edges.edge_ids.len(), 1);

    let replay_entities = service
        .ontology_upsert_entities(OntologyUpsertEntitiesRequest {
            ontology_path: ontology_path.clone(),
            entities: vec![
                OntologyEntityInput {
                    kind: OntologyEntityKind::Document,
                    attrs: {
                        let mut attrs = BTreeMap::new();
                        attrs.insert(
                            "source_ref".to_string(),
                            "2023/WF--BH-CHK--2023-01--statement.pdf".to_string(),
                        );
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
            ],
        })
        .expect("replay should succeed");

    assert_eq!(replay_entities.inserted_count, 0);
    assert_eq!(replay_entities.entity_ids, entities.entity_ids);
}

// ONTO-01 (D-03): edge upsert deterministically rejects missing from/to references.
#[test]
fn onto_01_missing_ref_rejected_deterministically() {
    let service = service();
    let tmp = tempdir().expect("tempdir");
    let ontology_path = tmp.path().join("ontology.json");

    let err = service
        .ontology_upsert_edges(OntologyUpsertEdgesRequest {
            ontology_path,
            edges: vec![OntologyEdgeInput {
                from: "missing-document".to_string(),
                to: "missing-transaction".to_string(),
                relation: "documents_transaction".to_string(),
                provenance: BTreeMap::new(),
            }],
        })
        .expect_err("invalid edge should fail");

    assert_eq!(
        err.to_string(),
        "invalid input: missing_ref: edge endpoints must reference existing entities"
    );
}

// ONTO-02 (D-03): deterministic traversal from document to transaction to evidence/tax nodes.
#[test]
fn onto_02_relationship_query_returns_ordered_document_chain() {
    let service = service();
    let tmp = tempdir().expect("tempdir");
    let ontology_path = tmp.path().join("ontology.json");

    let entities = service
        .ontology_upsert_entities(OntologyUpsertEntitiesRequest {
            ontology_path: ontology_path.clone(),
            entities: vec![
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
                    kind: OntologyEntityKind::EvidenceReference,
                    attrs: {
                        let mut attrs = BTreeMap::new();
                        attrs.insert("rkyv_ref".to_string(), "wf-ctx.rkyv".to_string());
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
            ],
        })
        .expect("entities should upsert");

    let doc = entities.entity_ids[0].clone();
    let tx = entities.entity_ids[1].clone();
    let evidence = entities.entity_ids[2].clone();
    let tax = entities.entity_ids[3].clone();

    service
        .ontology_upsert_edges(OntologyUpsertEdgesRequest {
            ontology_path: ontology_path.clone(),
            edges: vec![
                OntologyEdgeInput {
                    from: doc.clone(),
                    to: tx.clone(),
                    relation: "documents_transaction".to_string(),
                    provenance: BTreeMap::new(),
                },
                OntologyEdgeInput {
                    from: tx.clone(),
                    to: evidence.clone(),
                    relation: "links_evidence".to_string(),
                    provenance: BTreeMap::new(),
                },
                OntologyEdgeInput {
                    from: tx.clone(),
                    to: tax.clone(),
                    relation: "links_tax_category".to_string(),
                    provenance: BTreeMap::new(),
                },
            ],
        })
        .expect("edges should upsert");

    let chain = service
        .ontology_query_path_tool(OntologyQueryPathRequest {
            ontology_path,
            from_entity_id: doc.clone(),
            max_depth: Some(4),
        })
        .expect("path query should succeed");

    assert_eq!(
        chain
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>(),
        vec![doc, tx, evidence, tax]
    );

    assert_eq!(
        chain
            .edges
            .iter()
            .map(|edge| (edge.from.clone(), edge.to.clone(), edge.relation.clone()))
            .collect::<Vec<_>>(),
        vec![
            (
                chain.nodes[0].id.clone(),
                chain.nodes[1].id.clone(),
                "documents_transaction".to_string(),
            ),
            (
                chain.nodes[1].id.clone(),
                chain.nodes[2].id.clone(),
                "links_evidence".to_string(),
            ),
            (
                chain.nodes[1].id.clone(),
                chain.nodes[3].id.clone(),
                "links_tax_category".to_string(),
            ),
        ]
    );
}
