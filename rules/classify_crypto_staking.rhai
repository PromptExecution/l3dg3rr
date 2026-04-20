// rules/classify_crypto_staking.rhai
//
// Classification rule: CryptoIncome
//
// Rev. Rul. 2023-14 — staking rewards received by a cash-method taxpayer are
// included in gross income at the fair market value of the cryptocurrency at
// the time the taxpayer gains dominion and control over the rewards. Staking
// rewards are ordinary income, not capital gain.
//
// This overrules the Jarrett district court decision and aligns IRS treatment
// with mining income under Rev. Rul. 2023-14.
//
// Signals (confidence 0.88):
//   "staking reward", "validator reward", "proof of stake", "staking income",
//   "DeFi yield", "liquidity mining", "yield farming"
//
// Airdrop signal (confidence 0.80 — harder to value, timing of dominion unclear):
//   "airdrop"
//
// All events require review: true — taxpayer must determine FMV on receipt
// date to compute ordinary income amount.
//
// See: Rev. Rul. 2023-14; IRS Notice 2014-21; IRC §61(a)(1).

fn classify(tx) {
    let description = "";

    if tx.contains("description") {
        description = tx["description"];
    }

    let desc_lower = description.to_lower();

    // Staking / DeFi income signals — ordinary income at FMV on receipt.
    let staking_signal = desc_lower.contains("staking reward")
        || desc_lower.contains("validator reward")
        || desc_lower.contains("proof of stake")
        || desc_lower.contains("staking income")
        || desc_lower.contains("defi yield")
        || desc_lower.contains("liquidity mining")
        || desc_lower.contains("yield farming");

    // Airdrop — present but harder to value; separate confidence tier.
    let airdrop_signal = desc_lower.contains("airdrop");

    if !staking_signal && !airdrop_signal {
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no staking/DeFi/airdrop income keyword in description"
        };
    }

    // Airdrop gets lower confidence due to valuation uncertainty (dominion
    // and control timing can be ambiguous for new token distributions).
    if airdrop_signal && !staking_signal {
        return #{
            category:   "CryptoIncome",
            confidence: 0.80,
            review:     true,
            reason:     "Rev. Rul. 2023-14: airdrop — likely ordinary income at FMV on receipt; verify dominion and control date and valuation"
        };
    }

    #{
        category:   "CryptoIncome",
        confidence: 0.88,
        review:     true,
        reason:     "Rev. Rul. 2023-14: staking rewards taxable as ordinary income at FMV on receipt"
    }
}
