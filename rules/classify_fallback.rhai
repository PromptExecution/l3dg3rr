// rules/classify_fallback.rhai
//
// Classification rule: Unclassified (catch-all fallback)
//
// This rule matches every transaction unconditionally. It is the final rule
// in a chain — load it last. Any transaction that reaches this rule has not
// been classified by a more specific rule file and MUST be surfaced for
// manual CPA review.
//
// confidence: 0.0 — no classification evidence gathered.
// review: true  — unconditional; all unclassified transactions need review.
//
// The Rust engine also applies its own review_threshold, but this rule sets
// review: true explicitly so the flag is raised regardless of threshold.

fn classify(tx) {
    // Attempt to echo back the description for a useful audit reason string.
    // Falls back gracefully if the field is missing.
    let description = "";
    if tx.contains("description") {
        description = tx["description"];
    }

    let reason = if description != "" {
        "no rule matched — description: " + description
    } else {
        "no rule matched — transaction has no description"
    };

    #{
        category:   "Unclassified",
        confidence: 0.0,
        review:     true,
        reason:     reason
    }
}
