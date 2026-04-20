// rules/classify_fbar.rhai
//
// Classification rule: ForeignIncome / FBAR signal detection
//
// 31 USC §5314 — persons with a financial interest in, or signature authority
// over, a foreign financial account must file FinCEN Form 114 (FBAR) if the
// aggregate value of ALL foreign financial accounts exceeded $10,000 at any
// point during the calendar year.
//
// This rule detects individual transactions that indicate FBAR obligation and
// flags them for CPA review. It cannot determine the aggregate balance; that
// requires cross-account reconciliation outside this rule file.
//
// Foreign account signals:
//   account_id contains: "HSBC", "SWIFT", "IBAN", "BIC"
//   description contains: "wire transfer", "international transfer",
//                         "SEPA", "foreign bank"
//
// Review trigger: abs(amount) > $9,000 — approaching the $10,000 aggregate
// threshold; CPA must check aggregate balance across all foreign accounts.
//
// The reason string always includes the FBAR threshold citation regardless of
// review status so the audit trail is unambiguous.
//
// See: 31 USC §5314; 31 CFR §1010.350; FinCEN Form 114.

fn classify(tx) {
    let account_id  = "";
    let description = "";
    let amount_str  = "0";

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

    let desc_lower    = description.to_lower();
    let account_lower = account_id.to_lower();

    // Foreign account / institution signals.
    let foreign_account = account_id.contains("HSBC")
        || account_id.contains("SWIFT")
        || account_id.contains("IBAN")
        || account_id.contains("BIC");

    // Foreign transaction description signals.
    let foreign_desc = desc_lower.contains("wire transfer")
        || desc_lower.contains("international transfer")
        || desc_lower.contains("sepa")
        || desc_lower.contains("foreign bank");

    if !foreign_account && !foreign_desc {
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no FBAR foreign account or description signal"
        };
    }

    let amount_f   = parse_float(amount_str);
    let abs_amount = if amount_f < 0.0 { -amount_f } else { amount_f };

    // Near-threshold review: individual transaction > $9,000 may push aggregate
    // over the $10,000 annual FBAR filing threshold.
    let needs_review = abs_amount > 9000.0;

    let signal_desc = if foreign_account && foreign_desc {
        "foreign account ID and description keyword matched"
    } else if foreign_account {
        "foreign account ID matched (HSBC/SWIFT/IBAN/BIC)"
    } else {
        "foreign transaction description keyword matched (wire/SEPA/international)"
    };

    #{
        category:   "ForeignIncome",
        confidence: 0.82,
        review:     needs_review,
        reason:     signal_desc + " — FBAR: check aggregate balance against $10,000 annual threshold (31 USC §5314)"
    }
}
