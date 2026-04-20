// rules/classify_foreign_income.rhai
//
// Classification rule: ForeignIncome
//
// Signals used:
//   - account_id contains "HSBC" → strong foreign account indicator
//   - description contains "DE" (Germany) → foreign-source income keyword
//
// Amount handling: tx.amount is a STRING (rust_decimal::Decimal bridging
// limitation — no custom type registered). Use parse_float() for numeric
// comparisons only; never store or return monetary values as floats.
//
// Review trigger: amount parsed as float > 10000 → high-value foreign
// transfer, may implicate FBAR reporting threshold. Sets review: true so
// the Rust engine emits a ReviewFlag for CPA inspection.
//
// Confidence: 0.90 — foreign account + description signal is strong but not
// verified against a canonical foreign-entity registry.

fn classify(tx) {
    // Guard: if required fields are missing or empty, return Unclassified
    // rather than panicking. Rhai map access returns () (unit) for missing
    // keys, which is falsy in string contains checks — use explicit guard.
    let account_id = "";
    let description = "";
    let amount_str = "0";

    if tx.contains("account_id") {
        account_id = tx["account_id"];
    }
    if tx.contains("description") {
        description = tx["description"];
    }
    if tx.contains("amount") {
        amount_str = tx["amount"];
        if amount_str == "" {
            amount_str = "0";
        }
    }

    // Determine whether this transaction looks like foreign income.
    // "HSBC" in account_id is a strong indicator; "DE" in description
    // (German employer wire, DE entity references) is a secondary signal.
    let is_foreign_account = account_id.contains("HSBC");
    let is_foreign_desc    = description.contains("DE") || description.contains("Germany")
                             || description.contains("EUR") || description.contains("foreign");

    if !is_foreign_account && !is_foreign_desc {
        // No foreign signal — this rule does not apply.
        // Return Unclassified so the engine continues to the next rule file.
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no foreign account or description signal"
        };
    }

    // Numeric threshold check for FBAR-adjacent review trigger.
    // parse_float() is a Rhai 1.24.0 built-in. The amount string may be
    // negative (debit) — use abs() for threshold comparison.
    let amount_f = parse_float(amount_str);
    let abs_amount = if amount_f < 0.0 { -amount_f } else { amount_f };
    let needs_review = abs_amount > 10000.0;

    let reason = if is_foreign_account && is_foreign_desc {
        "foreign account ID and description keyword matched"
    } else if is_foreign_account {
        "foreign account ID matched (HSBC)"
    } else {
        "foreign description keyword matched"
    };

    #{
        category:   "ForeignIncome",
        confidence: 0.90,
        review:     needs_review,
        reason:     reason
    }
}
