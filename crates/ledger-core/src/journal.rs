use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::ingest::{deterministic_tx_id, TransactionInput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalTransaction {
    pub date: String,
    pub payee: String,
    pub narration: String,
    pub asset_account: String,
    pub counterparty_account: String,
    pub amount: String,
    pub currency: String,
    pub tx_id: String,
    pub source_ref: String,
}

impl JournalTransaction {
    pub fn from_input(row: &TransactionInput) -> Self {
        let tx_id = deterministic_tx_id(row);
        Self {
            date: row.date.clone(),
            payee: "Imported".to_string(),
            narration: row.description.clone(),
            asset_account: format!("Assets:Bank:{}", row.account_id),
            counterparty_account: "Equity:Suspense:Imported".to_string(),
            amount: row.amount.clone(),
            currency: "USD".to_string(),
            tx_id,
            source_ref: row.source_ref.clone(),
        }
    }

    pub fn to_beancount_entry(&self) -> String {
        let inverse = invert_amount(&self.amount);
        format!(
            "{} * \"{}\" \"{}\"\n  txid: \"{}\"\n  source_ref: \"{}\"\n  {} {} {}\n  {} {} {}\n",
            self.date,
            self.payee,
            self.narration.replace('"', "'"),
            self.tx_id,
            self.source_ref.replace('"', "'"),
            self.asset_account,
            self.amount,
            self.currency,
            self.counterparty_account,
            inverse,
            self.currency
        )
    }
}

pub fn append_entries(path: &Path, entries: &[JournalTransaction]) -> std::io::Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    for entry in entries {
        file.write_all(entry.to_beancount_entry().as_bytes())?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

fn invert_amount(amount: &str) -> String {
    let trimmed = amount.trim();
    if let Some(rest) = trimmed.strip_prefix('-') {
        rest.to_string()
    } else if let Some(rest) = trimmed.strip_prefix('+') {
        format!("-{}", rest)
    } else {
        format!("-{}", trimmed)
    }
}
