use std::path::Path;

use rust_xlsxwriter::Workbook;
use serde::{Deserialize, Serialize};

pub const REQUIRED_SHEETS: &[&str] = &[
    "META.config",
    "ACCT.registry",
    "CAT.taxonomy",
    "SCHED.C",
    "SCHED.D",
    "SCHED.E",
    "FBAR.accounts",
    "FLAGS.open",
    "FLAGS.resolved",
    "AUDIT.log",
];

pub fn initialize_workbook(path: &Path) -> Result<(), rust_xlsxwriter::XlsxError> {
    let mut workbook = Workbook::new();
    for sheet_name in REQUIRED_SHEETS {
        workbook.add_worksheet().set_name(*sheet_name)?;
    }
    workbook.save(path)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxProjectionRow {
    pub tx_id: String,
    pub account_id: String,
    pub date: String,
    pub amount: String,
    pub description: String,
    pub source_ref: String,
}

pub fn materialize_tx_projection(
    path: &Path,
    rows: &[TxProjectionRow],
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let mut workbook = Workbook::new();
    for sheet_name in REQUIRED_SHEETS {
        workbook.add_worksheet().set_name(*sheet_name)?;
    }

    let mut grouped = std::collections::BTreeMap::<String, Vec<&TxProjectionRow>>::new();
    for row in rows {
        grouped.entry(row.account_id.clone()).or_default().push(row);
    }

    for (account_id, account_rows) in grouped {
        let sheet_name = format!("TX.{}", account_id);
        let worksheet = workbook.add_worksheet().set_name(sheet_name)?;
        worksheet.write_string(0, 0, "tx_id")?;
        worksheet.write_string(0, 1, "date")?;
        worksheet.write_string(0, 2, "amount")?;
        worksheet.write_string(0, 3, "description")?;
        worksheet.write_string(0, 4, "source_ref")?;

        for (idx, row) in account_rows.into_iter().enumerate() {
            let r = (idx + 1) as u32;
            worksheet.write_string(r, 0, &row.tx_id)?;
            worksheet.write_string(r, 1, &row.date)?;
            worksheet.write_string(r, 2, &row.amount)?;
            worksheet.write_string(r, 3, &row.description)?;
            worksheet.write_string(r, 4, &row.source_ref)?;
        }
    }

    workbook.save(path)
}
