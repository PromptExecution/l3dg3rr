// rules/classify_schedule_d.rhai
//
// Classification rule: CapitalGain / CapitalLoss (Schedule D)
//
// IRC §1222(1) — short-term capital gain/loss: property held ≤ 12 months,
// taxed as ordinary income at the taxpayer's marginal rate.
// IRC §1222(3) — long-term capital gain/loss: property held > 12 months,
// taxed at preferential rates (0/15/20%).
//
// Signals (confidence 0.85):
//   "sale proceeds", "stock sale", "equity sale", "covered call",
//   "option exercise", "brokerage sale", "ETF sale"
//
// Loss detection: amount negative AND gain signal → CapitalLoss (confidence 0.85).
//
// Short-term review trigger: description contains "short" or "< 1 year" →
// review: true because STCG is taxed as ordinary income and commonly missed.
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

    // Capital event keywords — securities disposal signal.
    let gain_signal = desc_lower.contains("sale proceeds")
        || desc_lower.contains("stock sale")
        || desc_lower.contains("equity sale")
        || desc_lower.contains("covered call")
        || desc_lower.contains("option exercise")
        || desc_lower.contains("brokerage sale")
        || desc_lower.contains("etf sale");

    if !gain_signal {
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no Schedule D capital event keyword in description"
        };
    }

    let amount_f = parse_float(amount_str);

    // Short-term holding period detected → flag for ordinary income treatment.
    let short_term = desc_lower.contains("short") || desc_lower.contains("< 1 year");

    if amount_f < 0.0 {
        // Negative proceeds indicate a realized loss.
        return #{
            category:   "CapitalLoss",
            confidence: 0.85,
            review:     short_term,
            reason:     "IRC §1222: capital loss signal — negative sale proceeds"
        };
    }

    // Positive proceeds → capital gain.
    #{
        category:   "CapitalGain",
        confidence: 0.85,
        review:     short_term,
        reason:     "IRC §1222(3) long-term / §1222(1) short-term: capital gain signal — sale proceeds keyword matched"
    }
}
