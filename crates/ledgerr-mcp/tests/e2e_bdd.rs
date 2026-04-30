mod common;

use calamine::Reader;
use ledger_core::ingest::TransactionInput;
use ledgerr_mcp::{GetRawContextRequest, IngestPdfRequest, TurboLedgerService, TurboLedgerTools};

fn service() -> TurboLedgerService {
    let workbook_path = common::unique_workbook_path("e2e-bdd");
    TurboLedgerService::from_manifest_str(&common::manifest_for_workbook(&workbook_path, 2023))
        .expect("manifest should parse")
}

#[test]
fn bdd_e2e_ingest_statement_and_retrieve_evidence_context() {
    // Given a contract-valid statement file and extracted rows.
    let svc = service();
    let tmp = tempfile::tempdir().expect("tmpdir");
    let journal_path = tmp.path().join("ledger.beancount");
    let workbook_path = tmp.path().join("tax-ledger.xlsx");
    let source_ref = tmp.path().join("WF--BH-CHK--2023-01--statement.rkyv");

    let req = IngestPdfRequest {
        pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
        journal_path: journal_path.clone(),
        workbook_path: workbook_path.clone(),
        ontology_path: None,
        raw_context_bytes: Some(b"raw-evidence-bytes".to_vec()),
        extracted_rows: vec![TransactionInput {
            account_id: "WF-BH-CHK".to_string(),
            date: "2023-01-15".to_string(),
            amount: "-42.11".to_string(),
            description: "Coffee Shop".to_string(),
            source_ref: source_ref.display().to_string(),
        }],
    };

    // When the agent-facing ingest tool executes.
    let ingest = svc.ingest_pdf(req).expect("ingest should succeed");

    // Then deterministic tx ids are returned and artifacts are materialized.
    assert_eq!(ingest.inserted_count, 1);
    assert_eq!(ingest.tx_ids.len(), 1);

    let journal = std::fs::read_to_string(&journal_path).expect("journal exists");
    assert!(journal.contains("txid:"));
    assert!(journal.contains("source_ref:"));

    let workbook = calamine::open_workbook_auto(&workbook_path).expect("workbook exists");
    assert!(workbook.sheet_names().iter().any(|s| s == "TX.WF-BH-CHK"));

    // And raw context is retrievable through MCP by rkyv reference.
    let raw = svc
        .get_raw_context(GetRawContextRequest {
            rkyv_ref: source_ref,
        })
        .expect("raw context read should succeed");
    assert_eq!(raw.bytes, b"raw-evidence-bytes");
}

#[test]
fn bdd_e2e_reingest_is_idempotent_across_mcp_and_artifacts() {
    // Given the same source statement is ingested twice.
    let svc = service();
    let tmp = tempfile::tempdir().expect("tmpdir");
    let journal_path = tmp.path().join("ledger.beancount");
    let workbook_path = tmp.path().join("tax-ledger.xlsx");
    let source_ref = tmp.path().join("WF--BH-CHK--2023-01--statement.rkyv");

    let build_req = || IngestPdfRequest {
        pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
        journal_path: journal_path.clone(),
        workbook_path: workbook_path.clone(),
        ontology_path: None,
        raw_context_bytes: Some(b"raw-evidence-bytes".to_vec()),
        extracted_rows: vec![TransactionInput {
            account_id: "WF-BH-CHK".to_string(),
            date: "2023-01-15".to_string(),
            amount: "-42.11".to_string(),
            description: "Coffee Shop".to_string(),
            source_ref: source_ref.display().to_string(),
        }],
    };

    // When ingest is called twice with identical inputs.
    let first = svc.ingest_pdf(build_req()).expect("first ingest");
    let second = svc.ingest_pdf(build_req()).expect("second ingest");

    // Then first insert succeeds and second inserts nothing.
    assert_eq!(first.inserted_count, 1);
    assert_eq!(second.inserted_count, 0);
    assert_eq!(first.tx_ids.len(), 1);
    assert!(second.tx_ids.is_empty());

    // And journal remains single-entry for the transaction.
    let journal = std::fs::read_to_string(&journal_path).expect("journal exists");
    assert_eq!(journal.matches("txid:").count(), 1);
}

#[test]
fn bdd_e2e_rejects_non_contract_pdf_filename() {
    // Given a statement row that is otherwise valid.
    let svc = service();
    let tmp = tempfile::tempdir().expect("tmpdir");

    let req = IngestPdfRequest {
        pdf_path: "not-contract.pdf".to_string(),
        journal_path: tmp.path().join("ledger.beancount"),
        workbook_path: tmp.path().join("tax-ledger.xlsx"),
        ontology_path: None,
        raw_context_bytes: Some(b"ctx".to_vec()),
        extracted_rows: vec![TransactionInput {
            account_id: "WF-BH-CHK".to_string(),
            date: "2023-01-15".to_string(),
            amount: "-42.11".to_string(),
            description: "Coffee Shop".to_string(),
            source_ref: tmp
                .path()
                .join("WF--BH-CHK--2023-01--statement.rkyv")
                .display()
                .to_string(),
        }],
    };

    // When ingest is attempted with a filename that violates the contract.
    let err = svc.ingest_pdf(req).expect_err("should reject filename");

    // Then the API returns an invalid-input error for early correction.
    assert!(err.to_string().contains("invalid input"));
}

#[test]
fn bdd_e2e_allows_missing_raw_bytes_if_source_ref_already_exists() {
    // Given an existing source_ref artifact created by a previous extraction pass.
    let svc = service();
    let tmp = tempfile::tempdir().expect("tmpdir");
    let source_ref = tmp.path().join("WF--BH-CHK--2023-01--statement.rkyv");
    std::fs::write(&source_ref, b"existing-context").expect("seed source ref");

    let req = IngestPdfRequest {
        pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
        journal_path: tmp.path().join("ledger.beancount"),
        workbook_path: tmp.path().join("tax-ledger.xlsx"),
        ontology_path: None,
        raw_context_bytes: None,
        extracted_rows: vec![TransactionInput {
            account_id: "WF-BH-CHK".to_string(),
            date: "2023-01-15".to_string(),
            amount: "-42.11".to_string(),
            description: "Coffee Shop".to_string(),
            source_ref: source_ref.display().to_string(),
        }],
    };

    // When ingest runs with no raw bytes.
    let ingest = svc
        .ingest_pdf(req)
        .expect("ingest should accept existing source ref");

    // Then the transaction still ingests and prior context remains readable.
    assert_eq!(ingest.inserted_count, 1);
    let raw = svc
        .get_raw_context(GetRawContextRequest {
            rkyv_ref: source_ref,
        })
        .expect("existing context readable");
    assert_eq!(raw.bytes, b"existing-context");
}

#[test]
fn bdd_e2e_requires_raw_bytes_when_source_ref_missing() {
    // Given a source_ref path that does not exist yet.
    let svc = service();
    let tmp = tempfile::tempdir().expect("tmpdir");
    let source_ref = tmp.path().join("WF--BH-CHK--2023-01--statement.rkyv");

    let req = IngestPdfRequest {
        pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
        journal_path: tmp.path().join("ledger.beancount"),
        workbook_path: tmp.path().join("tax-ledger.xlsx"),
        ontology_path: None,
        raw_context_bytes: None,
        extracted_rows: vec![TransactionInput {
            account_id: "WF-BH-CHK".to_string(),
            date: "2023-01-15".to_string(),
            amount: "-42.11".to_string(),
            description: "Coffee Shop".to_string(),
            source_ref: source_ref.display().to_string(),
        }],
    };

    // When ingest attempts to materialize context without bytes.
    let err = svc.ingest_pdf(req).expect_err("should require raw bytes");

    // Then the API fails clearly so callers can retry with required payload.
    assert!(err
        .to_string()
        .contains("raw_context_bytes required when source_ref file does not exist"));
}
