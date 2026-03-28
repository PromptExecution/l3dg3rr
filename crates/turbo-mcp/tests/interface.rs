use turbo_mcp::{ListAccountsRequest, TurboLedgerService, TurboLedgerTools};

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
