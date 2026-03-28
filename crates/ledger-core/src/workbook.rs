use std::path::Path;

use rust_xlsxwriter::Workbook;

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
