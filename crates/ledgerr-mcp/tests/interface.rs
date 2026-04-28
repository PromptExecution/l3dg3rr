mod common;

use ledger_core::ingest::TransactionInput;
use ledgerr_mcp::{
    GetRawContextRequest, IngestPdfRequest, IngestStatementRowsRequest, ListAccountsRequest,
    TurboLedgerService, TurboLedgerTools,
};

#[test]
fn list_accounts_is_stable_and_obvious() {
    let manifest = format!(
        "{}\n[accounts]\nWF-BH-CHK = {{ institution = \"Wells Fargo\", type = \"checking\", currency = \"USD\" }}\nCB-BTC = {{ institution = \"Coinbase\", type = \"exchange\", currency = \"USD\" }}\n",
        common::manifest_for_workbook(&common::unique_workbook_path("interface-accounts"), 2023)
    );

    let service = TurboLedgerService::from_manifest_str(&manifest).unwrap();
    let mut accounts = service.list_accounts().unwrap();
    accounts.sort_by(|a, b| a.account_id.cmp(&b.account_id));

    assert_eq!(accounts[0].account_id, "CB-BTC");
    assert_eq!(accounts[1].account_id, "WF-BH-CHK");
}

#[test]
fn list_accounts_tool_contract_is_explicit() {
    let manifest = format!(
        "{}\n[accounts]\nWF-BH-CHK = {{ institution = \"Wells Fargo\", type = \"checking\", currency = \"USD\" }}\n",
        common::manifest_for_workbook(
            &common::unique_workbook_path("interface-accounts-tool"),
            2023
        )
    );

    let service = TurboLedgerService::from_manifest_str(&manifest).unwrap();
    let response = service.list_accounts_tool(ListAccountsRequest).unwrap();

    assert_eq!(response.accounts.len(), 1);
    assert_eq!(response.accounts[0].account_id, "WF-BH-CHK");
}

#[test]
fn preflight_rejects_non_contract_filename() {
    let manifest =
        common::manifest_for_workbook(&common::unique_workbook_path("interface-preflight"), 2023);

    let service = TurboLedgerService::from_manifest_str(&manifest).unwrap();
    let err = service
        .validate_source_filename("bad-name.pdf")
        .unwrap_err();

    assert!(err.to_string().contains("invalid input"));
}

#[test]
fn ingest_statement_rows_writes_git_friendly_journal_once() {
    let manifest =
        common::manifest_for_workbook(&common::unique_workbook_path("interface-ingest"), 2023);
    let service = TurboLedgerService::from_manifest_str(&manifest).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let journal_path = tmp.path().join("ledger.beancount");
    let workbook_path = tmp.path().join("tax-ledger.xlsx");

    let rows = vec![TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-15".to_string(),
        amount: "-42.11".to_string(),
        description: "Coffee Shop".to_string(),
        source_ref: "2023-taxes/WF--BH-CHK--2023-01--statement.rkyv".to_string(),
    }];

    let first = service
        .ingest_statement_rows(IngestStatementRowsRequest {
            journal_path: journal_path.clone(),
            workbook_path: workbook_path.clone(),
            ontology_path: None,
            rows: rows.clone(),
        })
        .unwrap();
    let second = service
        .ingest_statement_rows(IngestStatementRowsRequest {
            journal_path: journal_path.clone(),
            workbook_path,
            ontology_path: None,
            rows,
        })
        .unwrap();

    assert_eq!(first.inserted_count, 1);
    assert_eq!(second.inserted_count, 0);

    let content = std::fs::read_to_string(journal_path).unwrap();
    assert!(content.contains("txid:"));
}

#[test]
fn ingest_pdf_validates_filename_and_ingests_rows() {
    let manifest =
        common::manifest_for_workbook(&common::unique_workbook_path("interface-ingest-pdf"), 2023);
    let service = TurboLedgerService::from_manifest_str(&manifest).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let journal_path = tmp.path().join("ledger.beancount");
    let workbook_path = tmp.path().join("tax-ledger.xlsx");

    let response = service
        .ingest_pdf(IngestPdfRequest {
            pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
            journal_path,
            workbook_path,
            ontology_path: None,
            raw_context_bytes: Some(b"ctx".to_vec()),
            extracted_rows: vec![TransactionInput {
                account_id: "WF-BH-CHK".to_string(),
                date: "2023-01-15".to_string(),
                amount: "-42.11".to_string(),
                description: "Coffee Shop".to_string(),
                source_ref: tmp.path().join("ctx.rkyv").display().to_string(),
            }],
        })
        .unwrap();

    assert_eq!(response.inserted_count, 1);
    assert_eq!(response.tx_ids.len(), 1);
}

#[test]
fn get_raw_context_reads_rkyv_reference_bytes() {
    let manifest =
        common::manifest_for_workbook(&common::unique_workbook_path("interface-raw-context"), 2023);
    let service = TurboLedgerService::from_manifest_str(&manifest).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let rkyv_ref = tmp.path().join("sample.rkyv");
    std::fs::write(&rkyv_ref, b"rkyv-bytes").unwrap();

    let response = service
        .get_raw_context(GetRawContextRequest { rkyv_ref })
        .unwrap();
    assert_eq!(response.bytes, b"rkyv-bytes");
}
