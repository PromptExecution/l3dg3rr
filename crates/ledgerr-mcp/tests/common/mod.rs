use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn unique_workbook_path(label: &str) -> PathBuf {
    let suffix = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "ledgerr-{label}-{}-{suffix}.xlsx",
        std::process::id()
    ))
}

pub fn manifest_for_workbook(workbook_path: &Path, active_year: i32) -> String {
    format!(
        "[session]\nworkbook_path=\"{}\"\nactive_year={active_year}\n",
        workbook_path.display()
    )
}

#[allow(dead_code)]
pub fn stdio_test_manifest(label: &str) -> String {
    format!(
        "{}\n[accounts]\nWF-BH-CHK = {{ institution = \"Wells Fargo\", type = \"checking\", currency = \"USD\" }}\n",
        manifest_for_workbook(&unique_workbook_path(label), 2023)
    )
}
