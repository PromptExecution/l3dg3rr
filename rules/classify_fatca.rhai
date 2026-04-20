// rules/classify_fatca.rhai
//
// Classification rule: ForeignIncome / FATCA Form 8938 signal detection
//
// IRC §6038D — U.S. persons with specified foreign financial assets above the
// applicable threshold must attach Form 8938 to their annual return.
//
// Filing thresholds (single filers; married thresholds are 1.5× and 1.5×):
//   $50,000 at year-end OR $75,000 at any point during the year.
//   (Married filing jointly: $100,000 / $150,000.)
//
// This rule detects individual transactions that may signal FATCA disclosure
// obligations. Actual Form 8938 eligibility requires aggregate valuation
// across all specified foreign financial assets — this rule flags for review.
//
// Signals:
//   account_id contains: "HSBC", "SWIFT"
//   description contains: "foreign financial", "offshore", "foreign investment"
//
// AND abs(amount) > $25,000 → strong concern, review: true
// Lower amounts with same signal → confidence 0.65, review: false
//   (below single-transaction threshold concern)
//
// See: IRC §6038D; Treas. Reg. §1.6038D-2; IRS Form 8938 instructions.

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

    let desc_lower = description.to_lower();

    // FATCA-specific foreign financial asset signals.
    let fatca_account = account_id.contains("HSBC")
        || account_id.contains("SWIFT");

    let fatca_desc = desc_lower.contains("foreign financial")
        || desc_lower.contains("offshore")
        || desc_lower.contains("foreign investment");

    if !fatca_account && !fatca_desc {
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no FATCA foreign financial asset signal"
        };
    }

    let amount_f   = parse_float(amount_str);
    let abs_amount = if amount_f < 0.0 { -amount_f } else { amount_f };

    // High-value: materially relevant to Form 8938 thresholds.
    if abs_amount > 25000.0 {
        return #{
            category:   "ForeignIncome",
            confidence: 0.82,
            review:     true,
            reason:     "FATCA: evaluate Form 8938 requirement (IRC §6038D) — foreign financial asset signal with significant amount"
        };
    }

    // Lower amount — signal present but below practical threshold concern.
    #{
        category:   "ForeignIncome",
        confidence: 0.65,
        review:     false,
        reason:     "FATCA: foreign financial asset signal present — amount below $25,000 threshold concern (IRC §6038D)"
    }
}
