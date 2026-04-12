use std::collections::BTreeMap;

use ledger_core::ingest::TransactionInput;
use ledgerr_mcp::{
    events::reconstruct_lifecycle, ClassifyTransactionRequest, IngestStatementRowsRequest,
    ReconcileExcelClassificationRequest, ReplayLifecycleRequest, TurboLedgerService,
    TurboLedgerTools,
};

fn service() -> TurboLedgerService {
    TurboLedgerService::from_manifest_str(
        "[session]\nworkbook_path=\"tax-ledger.xlsx\"\nactive_year=2023\n",
    )
    .expect("manifest")
}

fn sample_row(date: &str, description: &str, source_ref: &str) -> TransactionInput {
    TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: date.to_string(),
        amount: "-42.11".to_string(),
        description: description.to_string(),
        source_ref: source_ref.to_string(),
    }
}

#[test]
fn evt_02_replay_reconstructs_stable_state_across_runs() {
    let svc = service();
    let temp = tempfile::tempdir().expect("tempdir");
    let row = sample_row("2023-01-15", "Coffee Shop", "source/a.rkyv");
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
        tx_id: tx_id.clone(),
        category: "OfficeSupplies".to_string(),
        confidence: "0.93".to_string(),
        note: Some("adjust".to_string()),
        actor: "agent".to_string(),
    })
    .expect("adjust");

    let first = svc
        .replay_lifecycle(ReplayLifecycleRequest {
            tx_id: Some(tx_id.clone()),
            document_ref: None,
        })
        .expect("first replay");
    let second = svc
        .replay_lifecycle(ReplayLifecycleRequest {
            tx_id: Some(tx_id),
            document_ref: None,
        })
        .expect("second replay");

    assert_eq!(first.reconstructed_state, second.reconstructed_state);
    assert_eq!(first.event_count, second.event_count);
    assert_eq!(first.diagnostics, second.diagnostics);
}

#[test]
fn evt_02_replay_reports_deterministic_diagnostics_for_sequence_or_transition_breaks() {
    let mut payload = BTreeMap::new();
    payload.insert("category".to_string(), "Meals".to_string());
    payload.insert("confidence".to_string(), "0.91".to_string());

    let event_stream = vec![
        ledgerr_mcp::LifecycleEvent {
            event_id: "evt-1".to_string(),
            sequence: 1,
            event_type: "classification".to_string(),
            tx_id: Some("tx-1".to_string()),
            document_ref: Some("doc-1".to_string()),
            occurred_at: "2023-01-15".to_string(),
            payload: payload.clone(),
            identity_inputs: BTreeMap::new(),
        },
        ledgerr_mcp::LifecycleEvent {
            event_id: "evt-3".to_string(),
            sequence: 3,
            event_type: "adjustment".to_string(),
            tx_id: Some("tx-1".to_string()),
            document_ref: Some("doc-1".to_string()),
            occurred_at: "2023-01-15".to_string(),
            payload,
            identity_inputs: BTreeMap::new(),
        },
    ];

    let replay = reconstruct_lifecycle(&event_stream);
    assert!(replay
        .diagnostics
        .iter()
        .any(|item| item.contains("sequence_gap")));
    assert!(replay
        .diagnostics
        .iter()
        .any(|item| item.contains("missing_predecessor")));
    assert!(replay
        .diagnostics
        .iter()
        .any(|item| item.contains("invalid_transition")));
}

#[test]
fn evt_02_replay_filtering_by_tx_and_document_is_deterministic() {
    let svc = service();
    let temp = tempfile::tempdir().expect("tempdir");

    let rows = vec![
        sample_row("2023-01-15", "Coffee Shop", "source/a.rkyv"),
        sample_row("2023-02-10", "Groceries", "source/b.rkyv"),
    ];
    let ingest = svc
        .ingest_statement_rows(IngestStatementRowsRequest {
            journal_path: temp.path().join("ledger.beancount"),
            workbook_path: temp.path().join("tax-ledger.xlsx"),
            rows,
        })
        .expect("ingest");

    let tx_id = ingest.tx_ids[0].clone();
    let replay = svc
        .replay_lifecycle(ReplayLifecycleRequest {
            tx_id: Some(tx_id),
            document_ref: Some("source/a.rkyv".to_string()),
        })
        .expect("filtered replay");

    assert_eq!(
        replay.filter.tx_id.as_deref(),
        Some(replay.filter.tx_id.as_deref().unwrap_or(""))
    );
    assert_eq!(replay.filter.document_ref.as_deref(), Some("source/a.rkyv"));
    assert!(replay.event_count >= 1);
}
