// rules/classify_crypto_trading.rhai
//
// Classification rule: Transfer / CryptoGain / CapitalLoss
//
// IRS Notice 2014-21 — cryptocurrency is treated as property for U.S. federal
// tax purposes. General tax principles applicable to property transactions apply.
// Each disposition is a taxable event; basis must be tracked per-unit.
//
// Rev. Proc. 2024-28 — UTXO (Unspent Transaction Output) lot-tracking relief;
// taxpayers may use a reasonable allocation method (FIFO, HIFO, specific ID)
// for determining basis on crypto dispositions.
//
// BUY signals → Transfer (basis establishment, not a taxable event):
//   "crypto purchase", "BTC buy", "ETH buy", "coin purchase", "exchange deposit"
//   confidence: 0.90, review: false
//
// SELL/EXCHANGE signals → capital event (taxable):
//   "crypto sale", "BTC sell", "ETH sell", "coin sale",
//   "exchange withdrawal", "crypto exchange"
//   amount positive → CryptoGain  (confidence 0.85, review: true)
//   amount negative → CapitalLoss (confidence 0.85, review: true)
//
// All sell/exchange events require review: true because the taxpayer must
// compute gain/loss using tracked basis, holding period, and fair market value.
//
// See: IRS Notice 2014-21; Rev. Proc. 2024-28; IRC §1001.

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

    // BUY signals — basis establishment, non-taxable transfer event.
    let buy_signal = desc_lower.contains("crypto purchase")
        || desc_lower.contains("btc buy")
        || desc_lower.contains("eth buy")
        || desc_lower.contains("coin purchase")
        || desc_lower.contains("exchange deposit");

    // SELL / EXCHANGE signals — taxable capital event.
    let sell_signal = desc_lower.contains("crypto sale")
        || desc_lower.contains("btc sell")
        || desc_lower.contains("eth sell")
        || desc_lower.contains("coin sale")
        || desc_lower.contains("exchange withdrawal")
        || desc_lower.contains("crypto exchange");

    if !buy_signal && !sell_signal {
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no cryptocurrency trading keyword in description"
        };
    }

    // BUY takes priority — basis-establishment transfers are not dispositions.
    if buy_signal {
        return #{
            category:   "Transfer",
            confidence: 0.90,
            review:     false,
            reason:     "IRS Notice 2014-21: crypto purchase — basis establishment (non-taxable transfer); track cost basis per Rev. Proc. 2024-28"
        };
    }

    // SELL / EXCHANGE — determine gain or loss from amount sign.
    let amount_f = parse_float(amount_str);

    if amount_f < 0.0 {
        return #{
            category:   "CapitalLoss",
            confidence: 0.85,
            review:     true,
            reason:     "IRS Notice 2014-21: crypto sale/exchange — capital loss; compute basis and holding period per Rev. Proc. 2024-28"
        };
    }

    #{
        category:   "CryptoGain",
        confidence: 0.85,
        review:     true,
        reason:     "IRS Notice 2014-21: crypto sale/exchange — capital gain; compute basis and holding period per Rev. Proc. 2024-28"
    }
}
