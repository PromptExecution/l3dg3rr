// rules/classify_schedule_c.rhai
//
// Classification rule: SelfEmployment / OfficeSupplies (Schedule C)
//
// IRC §162(a) — ordinary and necessary business expenses deductible in carrying
// on a trade or business. Income received and expenses paid in connection with
// self-employment activity are reported on Schedule C.
//
// Signals:
//   INCOME  (confidence 0.88): "invoice", "client payment", "consulting fee",
//           "freelance payment", "1099", "contract payment"
//   EXPENSE (confidence 0.82): "business expense", "office supply",
//           "software subscription", "professional development", "business meal"
//
// Amount handling: tx.amount is always a string — use parse_float() for
// numeric comparisons only. Never store or return monetary values as floats.
//
// Review triggers:
//   - Income amount > $5000
//   - Expense abs(amount) > $2500
//
// See: IRC §162(a); Treas. Reg. §1.162-1.

fn classify(tx) {
    let description = "";
    let amount_str  = "0";

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

    // Income keywords — strong Schedule C income signal.
    let income_match = desc_lower.contains("invoice")
        || desc_lower.contains("client payment")
        || desc_lower.contains("consulting fee")
        || desc_lower.contains("freelance payment")
        || desc_lower.contains("1099")
        || desc_lower.contains("contract payment");

    // Expense keywords — Schedule C deductible business expense signal.
    let expense_match = desc_lower.contains("business expense")
        || desc_lower.contains("office supply")
        || desc_lower.contains("software subscription")
        || desc_lower.contains("professional development")
        || desc_lower.contains("business meal");

    if !income_match && !expense_match {
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no Schedule C income or expense keyword in description"
        };
    }

    let amount_f   = parse_float(amount_str);
    let abs_amount = if amount_f < 0.0 { -amount_f } else { amount_f };

    // Negative amount with an expense keyword → OfficeSupplies deduction.
    if amount_f < 0.0 && expense_match {
        let needs_review = abs_amount > 2500.0;
        return #{
            category:   "OfficeSupplies",
            confidence: 0.75,
            review:     needs_review,
            reason:     "IRC §162(a): negative amount with business expense keyword — Schedule C deduction"
        };
    }

    // Expense keyword with positive amount (reimbursement / credit) or plain
    // income keyword.
    if income_match {
        let needs_review = amount_f > 5000.0;
        return #{
            category:   "SelfEmployment",
            confidence: 0.88,
            review:     needs_review,
            reason:     "IRC §162(a): Schedule C income keyword matched (invoice/consulting fee/1099/contract payment)"
        };
    }

    // Expense keyword, positive amount edge case.
    let needs_review = abs_amount > 2500.0;
    #{
        category:   "SelfEmployment",
        confidence: 0.82,
        review:     needs_review,
        reason:     "IRC §162(a): Schedule C business expense keyword matched"
    }
}
