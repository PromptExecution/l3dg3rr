use calamine::Reader;
use ledger_core::filename::StatementFilename;
use ledger_core::manifest::Manifest;
use ledger_core::workbook::{initialize_workbook, REQUIRED_SHEETS};

#[test]
fn parses_valid_statement_filename() {
    let parsed = StatementFilename::parse("WF--BH-CHK--2023-01--statement.pdf").unwrap();
    assert_eq!(parsed.vendor, "WF");
    assert_eq!(parsed.account, "BH-CHK");
    assert_eq!(parsed.year, 2023);
    assert_eq!(parsed.month, 1);
    assert_eq!(parsed.doc_type, "statement");
}

#[test]
fn rejects_invalid_statement_filename_before_mutation() {
    assert!(StatementFilename::parse("WF-BH-CHK-2023-01-statement.pdf").is_err());
    assert!(StatementFilename::parse("WF--BH-CHK--2023--statement.pdf").is_err());
    assert!(StatementFilename::parse("WF--BH-CHK--2023-13--statement.pdf").is_err());
}

#[test]
fn accepts_reasonable_filename_variants_and_normalizes_output() {
    let parsed = StatementFilename::parse("wf--bh-chk--2023-01--Statement.PDF").unwrap();
    assert_eq!(parsed.vendor, "WF");
    assert_eq!(parsed.account, "BH-CHK");
    assert_eq!(parsed.doc_type, "statement");
}

#[test]
fn loads_manifest_and_lists_accounts_without_workbook_io() {
    let src = r#"
[session]
workbook_path = "tax-ledger.xlsx"
active_year = 2023

[accounts]
WF-BH-CHK = { institution = "Wells Fargo", type = "checking", currency = "USD" }
CB-BTC = { institution = "Coinbase", type = "exchange", currency = "USD" }
"#;

    let manifest = Manifest::parse(src).unwrap();
    let mut ids = manifest.list_account_ids();
    ids.sort();
    assert_eq!(ids, vec!["CB-BTC".to_string(), "WF-BH-CHK".to_string()]);
    assert_eq!(manifest.session.workbook_path, "tax-ledger.xlsx");
}

#[test]
fn initializes_workbook_with_required_sheet_names() {
    let dir = tempfile::tempdir().unwrap();
    let workbook_path = dir.path().join("tax-ledger.xlsx");

    initialize_workbook(&workbook_path).unwrap();

    let workbook = calamine::open_workbook_auto(&workbook_path).unwrap();
    let names = workbook.sheet_names().to_vec();

    for required in REQUIRED_SHEETS {
        assert!(
            names.iter().any(|name| name == required),
            "missing {required}"
        );
    }
}
