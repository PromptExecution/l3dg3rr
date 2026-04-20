// rules/classify_self_employment.rhai
//
// Classification rule: SelfEmployment
//
// Signals used (description keyword matching):
//   Strong (confidence 0.85): "invoice", "client", "consulting"
//   Weak   (confidence 0.65): "freelance", "contract", "project"
//
// The weak tier intentionally falls below a typical 0.70 review threshold
// so that ambiguous matches surface for CPA inspection. The review field is
// set by this rule only when confidence would be < 0.70 due to weak signal;
// the Rust engine also applies its own review_threshold on top of this.
//
// Amount: string — parse_float() for numeric checks only. No f64 money math.
// No FBAR-style threshold trigger here; Schedule C flag is handled by the
// review: true return.

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

    // Lowercase the description once for case-insensitive matching.
    // Rhai 1.24.0 String has .to_lower() built-in.
    let desc_lower = description.to_lower();

    // Strong keywords → high confidence self-employment signal.
    let strong_match = desc_lower.contains("invoice")
        || desc_lower.contains("client")
        || desc_lower.contains("consulting");

    // Weak keywords → present but ambiguous; needs CPA review.
    let weak_match = desc_lower.contains("freelance")
        || desc_lower.contains("contract")
        || desc_lower.contains("project");

    if !strong_match && !weak_match {
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no self-employment keyword in description"
        };
    }

    let confidence = if strong_match { 0.85 } else { 0.65 };
    // review: true when confidence < 0.70, i.e. only weak signal matched.
    let needs_review = confidence < 0.70;

    let reason = if strong_match {
        "strong self-employment keyword matched (invoice/client/consulting)"
    } else {
        "weak self-employment keyword matched (freelance/contract/project) — review recommended"
    };

    #{
        category:   "SelfEmployment",
        confidence: confidence,
        review:     needs_review,
        reason:     reason
    }
}
