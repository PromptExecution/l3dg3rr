// rules/classify_au_cgt.rhai
//
// Classification rule: AuCgt (Australian Capital Gains Tax)
//
// Income Tax Assessment Act 1997 (ITAA 1997) s.115-A — the "CGT discount"
// method. An individual (not a company or trust) who has held a CGT asset for
// at least 12 months before the CGT event may reduce the capital gain by 50%
// before including it in assessable income.
//
// For US expats: AU CGT events must be reported on both the AU return AND the
// US return (subject to FTC or FEIE treatment). Double-tax treaty analysis
// (US-AU Treaty Art. 13) may alter the outcome.
//
// Signals:
//   description contains: "property sale", "real estate settlement",
//                         "shares sale", "investment property", "capital proceeds"
//   AND (account_id contains "AU" or "AUS") OR description contains "AUD"
//
// All matches require review: true — CPA must verify:
//   (1) acquisition date for CGT discount eligibility (s.115-A: > 12 months)
//   (2) US treatment of the same event under IRC §1001
//   (3) availability of foreign tax credits (IRC §901) for AU CGT paid
//
// See: ITAA 1997 s.115-A; US-Australia Tax Treaty Art. 13; IRC §§901, 1001.

fn classify(tx) {
    let account_id  = "";
    let description = "";

    if tx.contains("account_id") {
        account_id = tx["account_id"];
    }
    if tx.contains("description") {
        description = tx["description"];
    }

    let desc_lower = description.to_lower();
    let acct_upper = account_id.to_upper();

    // Capital proceeds / disposal keywords.
    let capital_signal = desc_lower.contains("property sale")
        || desc_lower.contains("real estate settlement")
        || desc_lower.contains("shares sale")
        || desc_lower.contains("investment property")
        || desc_lower.contains("capital proceeds");

    // Australian jurisdiction indicators.
    let au_jurisdiction = acct_upper.contains("AU")
        || acct_upper.contains("AUS")
        || desc_lower.contains("aud");

    if !capital_signal || !au_jurisdiction {
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no AU CGT capital proceeds signal or AU jurisdiction indicator"
        };
    }

    #{
        category:   "AuCgt",
        confidence: 0.87,
        review:     true,
        reason:     "ITAA 1997 s.115-A: 50% CGT discount may apply if held >12 months — verify acquisition date; also report on US return under IRC §1001"
    }
}
