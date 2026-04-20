// rules/classify_au_gst.rhai
//
// Classification rule: AuGst / ForeignIncome (Australian GST)
//
// A New Tax System (Goods and Services Tax) Act 1999 s.11-5 — an acquisition
// is a "creditable acquisition" if it is solely or partly for a creditable
// purpose (carrying on an enterprise). Input tax credits (ITCs) may be claimed
// for GST paid on creditable acquisitions.
//
// For a US expat with AU business activities:
//   - Creditable acquisition (expense with GST): AuGst — input tax credit claim
//   - GST-registered income (positive AU transaction): ForeignIncome — AU-sourced
//     income must be reported on the US return (IRC §61); review: true
//
// Signals:
//   account_id contains: "AU", "AUS", "ANZ", "CBA", "NAB", "WBC"
//   description contains: "GST", "tax invoice", "ABN", "BAS"
//
// See: A New Tax System (Goods and Services Tax) Act 1999 s.11-5;
//      IRC §61; IRS Publication 54 (Tax Guide for U.S. Citizens Abroad).

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

    let desc_upper = description.to_upper();
    let acct_upper = account_id.to_upper();

    // Australian bank/account signals.
    let au_account = acct_upper.contains("AU")
        || acct_upper.contains("AUS")
        || acct_upper.contains("ANZ")
        || acct_upper.contains("CBA")
        || acct_upper.contains("NAB")
        || acct_upper.contains("WBC");

    // GST transaction description signals.
    let gst_desc = desc_upper.contains("GST")
        || desc_upper.contains("TAX INVOICE")
        || desc_upper.contains("ABN")
        || desc_upper.contains("BAS");

    if !au_account && !gst_desc {
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no Australian GST account or description signal"
        };
    }

    let amount_f = parse_float(amount_str);

    // Negative amount with GST signal → creditable acquisition (ITC claimable).
    if amount_f < 0.0 {
        return #{
            category:   "AuGst",
            confidence: 0.85,
            review:     false,
            reason:     "A New Tax System (GST) Act 1999 s.11-5: creditable acquisition — GST expense eligible for input tax credit"
        };
    }

    // Positive amount with GST signal → AU-sourced income for US expat.
    // Must be reported on US return; also carries AU GST obligations.
    #{
        category:   "ForeignIncome",
        confidence: 0.78,
        review:     true,
        reason:     "A New Tax System (GST) Act 1999 s.11-5: AU-sourced income with GST signal — review for US expat foreign income reporting (IRC §61)"
    }
}
