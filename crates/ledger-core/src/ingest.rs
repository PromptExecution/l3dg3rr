use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionInput {
    pub account_id: String,
    pub date: String,
    pub amount: String,
    pub description: String,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestedTransaction {
    pub tx_id: String,
    pub source_ref: String,
}

#[derive(Debug, Default)]
pub struct IngestedLedger {
    seen: BTreeSet<String>,
}

impl IngestedLedger {
    pub fn ingest(&mut self, rows: &[TransactionInput]) -> Vec<IngestedTransaction> {
        let mut out = Vec::new();

        for row in rows {
            let tx_id = deterministic_tx_id(row);
            if self.seen.insert(tx_id.clone()) {
                out.push(IngestedTransaction {
                    tx_id,
                    source_ref: row.source_ref.clone(),
                });
            }
        }

        out
    }
}

pub fn deterministic_tx_id(row: &TransactionInput) -> String {
    let canonical = format!(
        "{}|{}|{}|{}",
        row.account_id.trim().to_ascii_uppercase(),
        row.date.trim(),
        row.amount.trim(),
        row.description.trim().to_ascii_lowercase(),
    );
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}
