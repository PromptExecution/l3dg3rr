mod common;

use std::collections::BTreeSet;

use ledger_core::ingest::TransactionInput;
use ledgerr_mcp::{
    ClassifyTransactionRequest, EventHistoryFilter, IngestStatementRowsRequest,
    ReconcileExcelClassificationRequest, TurboLedgerService, TurboLedgerTools,
};

fn service() -> TurboLedgerService {
    let workbook_path = common::unique_workbook_path("events-contract");
    TurboLedgerService::from_manifest_str(&common::manifest_for_workbook(&workbook_path, 2023))
        .expect("manifest")
}

fn sample_row(description: &str, amount: &str) -> TransactionInput {
    TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-15".to_string(),
        amount: amount.to_string(),
        description: description.to_string(),
        source_ref: "source/ctx.rkyv".to_string(),
    }
}

#[test]
fn evt_01_lifecycle_actions_append_typed_events_without_mutating_prior_entries() {
    let svc = service();
    let temp = tempfile::tempdir().expect("tempdir");
    let row = sample_row("Coffee Shop", "-42.11");
    let ingest = svc
        .ingest_statement_rows(IngestStatementRowsRequest {
            journal_path: temp.path().join("ledger.beancount"),
            workbook_path: temp.path().join("tax-ledger.xlsx"),
            rows: vec![row],
        })
        .expect("ingest");
    let tx_id = ingest.tx_ids[0].clone();

    svc.classify_transaction(ClassifyTransactionRequest {
        tx_id: tx_id.clone(),
        category: "Meals".to_string(),
        confidence: "0.91".to_string(),
        note: Some("classify".to_string()),
        actor: "agent".to_string(),
    })
    .expect("classify");

    svc.reconcile_excel_classification(ReconcileExcelClassificationRequest {
        tx_id: tx_id.clone(),
        category: "OfficeSupplies".to_string(),
        confidence: "0.88".to_string(),
        actor: "excel-user".to_string(),
        note: Some("reconcile".to_string()),
    })
    .expect("reconcile");

    svc.adjust_transaction(ClassifyTransactionRequest {
        tx_id,
        category: "OfficeSupplies".to_string(),
        confidence: "0.93".to_string(),
        note: Some("adjustment".to_string()),
        actor: "agent".to_string(),
    })
    .expect("adjust");

    let events = svc
        .event_history(EventHistoryFilter::default())
        .expect("event history");
    let event_types = events
        .events
        .iter()
        .map(|event| event.event_type.clone())
        .collect::<BTreeSet<_>>();

    assert!(event_types.contains("ingest"));
    assert!(event_types.contains("classification"));
    assert!(event_types.contains("reconciliation"));
    assert!(event_types.contains("adjustment"));

    let snapshot = events.events.clone();
    svc.adjust_transaction(ClassifyTransactionRequest {
        tx_id: snapshot[0].tx_id.clone().unwrap_or_default(),
        category: "OfficeSupplies".to_string(),
        confidence: "0.94".to_string(),
        note: Some("second-adjustment".to_string()),
        actor: "agent".to_string(),
    })
    .expect("append-only second adjustment");

    let after = svc
        .event_history(EventHistoryFilter::default())
        .expect("event history");
    assert_eq!(&after.events[..snapshot.len()], snapshot.as_slice());
}

#[test]
fn evt_01_replaying_same_operation_produces_stable_payload_and_identity_inputs() {
    let svc = service();
    let temp = tempfile::tempdir().expect("tempdir");
    let row = sample_row("Stationery", "-15.00");
    let ingest = svc
        .ingest_statement_rows(IngestStatementRowsRequest {
            journal_path: temp.path().join("ledger.beancount"),
            workbook_path: temp.path().join("tax-ledger.xlsx"),
            rows: vec![row],
        })
        .expect("ingest");
    let tx_id = ingest.tx_ids[0].clone();

    svc.classify_transaction(ClassifyTransactionRequest {
        tx_id: tx_id.clone(),
        category: "OfficeSupplies".to_string(),
        confidence: "0.80".to_string(),
        note: Some("first".to_string()),
        actor: "agent".to_string(),
    })
    .expect("first classify");
    svc.classify_transaction(ClassifyTransactionRequest {
        tx_id,
        category: "OfficeSupplies".to_string(),
        confidence: "0.80".to_string(),
        note: Some("first".to_string()),
        actor: "agent".to_string(),
    })
    .expect("second classify");

    let events = svc
        .event_history(EventHistoryFilter::default())
        .expect("history");
    let class_events = events
        .events
        .into_iter()
        .filter(|event| event.event_type == "classification")
        .collect::<Vec<_>>();

    assert!(class_events.len() >= 2);
    assert_eq!(class_events[0].payload, class_events[1].payload);
    assert_eq!(
        class_events[0].identity_inputs,
        class_events[1].identity_inputs
    );
}

#[test]
fn evt_01_store_contract_is_append_and_read_only() {
    use ledgerr_mcp::events::{InMemoryLifecycleEventStore, LifecycleEventStore};

    let mut store = InMemoryLifecycleEventStore::default();
    store
        .append_event("classification", None, None, Default::default())
        .expect("append");
    let events = store.list_events(Default::default()).expect("list");
    assert_eq!(events.events.len(), 1);
}
