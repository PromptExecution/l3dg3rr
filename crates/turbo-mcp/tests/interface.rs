use ledger_core::ingest::TransactionInput;
use turbo_mcp::{IngestStatementRowsRequest, ListAccountsRequest, TurboLedgerService, TurboLedgerTools};

#[test]
fn list_accounts_is_stable_and_obvious() {
    let manifest = r#"
[session]
workbook_path = "tax-ledger.xlsx"
active_year = 2023

[accounts]
WF-BH-CHK = { institution = "Wells Fargo", type = "checking", currency = "USD" }
CB-BTC = { institution = "Coinbase", type = "exchange", currency = "USD" }
"#;

    let service = TurboLedgerService::from_manifest_str(manifest).unwrap();
    let mut accounts = service.list_accounts().unwrap();
    accounts.sort_by(|a, b| a.account_id.cmp(&b.account_id));

    assert_eq!(accounts[0].account_id, "CB-BTC");
    assert_eq!(accounts[1].account_id, "WF-BH-CHK");
}

#[test]
fn list_accounts_tool_contract_is_explicit() {
    let manifest = r#"
[session]
workbook_path = "tax-ledger.xlsx"
active_year = 2023

[accounts]
WF-BH-CHK = { institution = "Wells Fargo", type = "checking", currency = "USD" }
"#;

    let service = TurboLedgerService::from_manifest_str(manifest).unwrap();
    let response = service.list_accounts_tool(ListAccountsRequest).unwrap();

    assert_eq!(response.accounts.len(), 1);
    assert_eq!(response.accounts[0].account_id, "WF-BH-CHK");
}

#[test]
fn preflight_rejects_non_contract_filename() {
    let manifest = r#"
[session]
workbook_path = "tax-ledger.xlsx"
active_year = 2023
"#;

    let service = TurboLedgerService::from_manifest_str(manifest).unwrap();
    let err = service.validate_source_filename("bad-name.pdf").unwrap_err();

    assert!(err.to_string().contains("invalid input"));
}

#[test]
fn ingest_statement_rows_writes_git_friendly_journal_once() {
    let manifest = r#"
[session]
workbook_path = "tax-ledger.xlsx"
active_year = 2023
"#;
    let service = TurboLedgerService::from_manifest_str(manifest).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let journal_path = tmp.path().join("ledger.beancount");

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
            rows: rows.clone(),
        })
        .unwrap();
    let second = service
        .ingest_statement_rows(IngestStatementRowsRequest {
            journal_path: journal_path.clone(),
            rows,
        })
        .unwrap();

    assert_eq!(first.inserted_count, 1);
    assert_eq!(second.inserted_count, 0);

    let content = std::fs::read_to_string(journal_path).unwrap();
    assert!(content.contains("txid:"));
}
