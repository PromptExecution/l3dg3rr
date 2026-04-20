// rules/classify_schedule_e.rhai
//
// Classification rule: RentalIncome (Schedule E)
//
// IRC §469(c)(1) — passive activity: rental activity is treated as passive
// per se regardless of participation level (unless the taxpayer is a
// real-estate professional under §469(c)(7)). Passive losses may only offset
// passive income; excess passive losses are suspended.
//
// Signals (confidence 0.87):
//   "rent", "rental income", "royalty", "K-1", "partnership distribution",
//   "LLC distribution", "S-corp distribution", "sublease"
//
// Review trigger: amount > $10,000 — passive loss limitation rules
// (IRC §469) may apply; CPA must verify participation and basis.
//
// Negative amount + rent signal → RentalIncome at confidence 0.80
// (could be a repair expense or maintenance offset — ambiguous).
//
// Amount handling: tx.amount is always a string — use parse_float() only.

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

    // Passive activity / rental income keywords.
    let rent_signal = desc_lower.contains("rent")
        || desc_lower.contains("rental income")
        || desc_lower.contains("royalty")
        || desc_lower.contains("k-1")
        || desc_lower.contains("partnership distribution")
        || desc_lower.contains("llc distribution")
        || desc_lower.contains("s-corp distribution")
        || desc_lower.contains("sublease");

    if !rent_signal {
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no Schedule E passive activity keyword in description"
        };
    }

    let amount_f   = parse_float(amount_str);
    let abs_amount = if amount_f < 0.0 { -amount_f } else { amount_f };

    // Negative amount with rent signal: possible repair/maintenance expense
    // offset — use lower confidence and flag for review.
    if amount_f < 0.0 {
        return #{
            category:   "RentalIncome",
            confidence: 0.80,
            review:     true,
            reason:     "IRC §469(c)(1): negative amount with rental keyword — possible expense offset; verify Schedule E treatment"
        };
    }

    let needs_review = amount_f > 10000.0;

    #{
        category:   "RentalIncome",
        confidence: 0.87,
        review:     needs_review,
        reason:     "IRC §469(c)(1): passive rental/royalty/K-1 distribution keyword matched"
    }
}
