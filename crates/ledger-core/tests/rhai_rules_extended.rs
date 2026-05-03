/// Integration tests for the extended `.rhai` rule files in `rules/`.
///
/// Each test invokes `ClassificationEngine::run_rule_from_file` — the same
/// path the production pipeline uses — exercising the real Rhai engine, real
/// file I/O, and real map extraction without mocking.
///
/// Run with: `cargo test -p ledger-core --test rhai_rules_extended`
///
/// # Path convention
/// `CARGO_MANIFEST_DIR` for an integration test points to the crate root
/// (`crates/ledger-core`); two parents up is the workspace root where
/// `rules/` lives.
use std::path::PathBuf;

use ledger_core::classify::{ClassificationEngine, SampleTransaction};

fn rule_path(filename: &str) -> PathBuf {
    let manifest =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo test");
    PathBuf::from(manifest)
        .parent() // crates/
        .expect("crates parent")
        .parent() // workspace root
        .expect("workspace root")
        .join("rules")
        .join(filename)
}

fn engine() -> ClassificationEngine {
    ClassificationEngine::default()
}

// ---------------------------------------------------------------------------
// classify_schedule_c.rhai — IRC §162(a)
// ---------------------------------------------------------------------------

#[test]
fn sc_01_income_keyword_classifies_self_employment() {
    // "consulting fee" is a strong income keyword → SelfEmployment, confidence 0.88,
    // amount 3000 is below the $5000 review trigger.
    let sample = SampleTransaction {
        tx_id: "sc-01".into(),
        account_id: "WF--BH-CHK--2024-02".into(),
        date: "2024-02-10".into(),
        amount: "3000.00".into(),
        description: "Consulting fee — Acme Corp Q1".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_c.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "SelfEmployment");
    assert!(
        (outcome.confidence - 0.88).abs() < f64::EPSILON,
        "expected 0.88, got {}",
        outcome.confidence
    );
    assert!(
        !outcome.needs_review,
        "amount 3000 is below $5000 review threshold"
    );
}

#[test]
fn sc_02_high_value_income_triggers_review() {
    // Income > $5000 must set review: true.
    let sample = SampleTransaction {
        tx_id: "sc-02".into(),
        account_id: "WF--BH-CHK--2024-03".into(),
        date: "2024-03-01".into(),
        amount: "7500.00".into(),
        description: "1099 contract payment — project closeout".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_c.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "SelfEmployment");
    assert!(outcome.needs_review, "amount > $5000 must trigger review");
}

#[test]
fn sc_03_negative_expense_keyword_classifies_office_supplies() {
    // Negative amount with "office supply" → OfficeSupplies at confidence 0.75.
    let sample = SampleTransaction {
        tx_id: "sc-03".into(),
        account_id: "WF--BH-CHK--2024-04".into(),
        date: "2024-04-05".into(),
        amount: "-150.00".into(),
        description: "Office supply purchase — printer paper and toner".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_c.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "OfficeSupplies");
    assert!(
        (outcome.confidence - 0.75).abs() < f64::EPSILON,
        "expected 0.75, got {}",
        outcome.confidence
    );
    assert!(
        !outcome.needs_review,
        "expense $150 is below $2500 review trigger"
    );
}

#[test]
fn sc_04_large_expense_triggers_review() {
    // Expense > $2500 abs must trigger review.
    let sample = SampleTransaction {
        tx_id: "sc-04".into(),
        account_id: "WF--BH-CHK--2024-05".into(),
        date: "2024-05-01".into(),
        amount: "-3000.00".into(),
        description: "Software subscription — annual professional development license".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_c.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "OfficeSupplies");
    assert!(outcome.needs_review, "expense > $2500 must trigger review");
}

#[test]
fn sc_05_no_keyword_unclassified() {
    let sample = SampleTransaction {
        tx_id: "sc-05".into(),
        account_id: "WF--BH-CHK--2024-01".into(),
        date: "2024-01-15".into(),
        amount: "200.00".into(),
        description: "Grocery store — Whole Foods".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_c.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "Unclassified");
}

// ---------------------------------------------------------------------------
// classify_schedule_d.rhai — IRC §1222
// ---------------------------------------------------------------------------

#[test]
fn sd_01_stock_sale_positive_classifies_capital_gain() {
    let sample = SampleTransaction {
        tx_id: "sd-01".into(),
        account_id: "FIDELITY--BH-BROK--2024-06".into(),
        date: "2024-06-15".into(),
        amount: "4500.00".into(),
        description: "Stock sale — AAPL shares".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_d.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "CapitalGain");
    assert!(
        (outcome.confidence - 0.85).abs() < f64::EPSILON,
        "expected 0.85, got {}",
        outcome.confidence
    );
    assert!(
        !outcome.needs_review,
        "no short-term signal — no review required"
    );
}

#[test]
fn sd_02_stock_sale_negative_classifies_capital_loss() {
    let sample = SampleTransaction {
        tx_id: "sd-02".into(),
        account_id: "FIDELITY--BH-BROK--2024-07".into(),
        date: "2024-07-01".into(),
        amount: "-1200.00".into(),
        description: "Stock sale — TSLA covered call expired worthless".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_d.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "CapitalLoss");
    assert!(
        (outcome.confidence - 0.85).abs() < f64::EPSILON,
        "expected 0.85, got {}",
        outcome.confidence
    );
}

#[test]
fn sd_03_short_term_signal_triggers_review() {
    // "short" in description → review: true (STCG taxed as ordinary income).
    let sample = SampleTransaction {
        tx_id: "sd-03".into(),
        account_id: "FIDELITY--BH-BROK--2024-08".into(),
        date: "2024-08-15".into(),
        amount: "800.00".into(),
        description: "Brokerage sale — short term gain ETF sale".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_d.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "CapitalGain");
    assert!(
        outcome.needs_review,
        "short-term indicator must trigger review"
    );
}

#[test]
fn sd_04_no_signal_unclassified() {
    let sample = SampleTransaction {
        tx_id: "sd-04".into(),
        account_id: "WF--BH-CHK--2024-01".into(),
        date: "2024-01-10".into(),
        amount: "500.00".into(),
        description: "Direct deposit payroll".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_d.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "Unclassified");
}

// ---------------------------------------------------------------------------
// classify_schedule_e.rhai — IRC §469(c)(1)
// ---------------------------------------------------------------------------

#[test]
fn se_01_rental_income_classifies_correctly() {
    let sample = SampleTransaction {
        tx_id: "se-01".into(),
        account_id: "WF--BH-CHK--2024-01".into(),
        date: "2024-01-05".into(),
        amount: "2200.00".into(),
        description: "Rental income — tenant January payment".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_e.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "RentalIncome");
    assert!(
        (outcome.confidence - 0.87).abs() < f64::EPSILON,
        "expected 0.87, got {}",
        outcome.confidence
    );
    assert!(
        !outcome.needs_review,
        "amount 2200 below $10000 review threshold"
    );
}

#[test]
fn se_02_high_value_rental_triggers_review() {
    let sample = SampleTransaction {
        tx_id: "se-02".into(),
        account_id: "WF--BH-CHK--2024-07".into(),
        date: "2024-07-01".into(),
        amount: "15000.00".into(),
        description: "K-1 partnership distribution — passive rental fund".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_e.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "RentalIncome");
    assert!(outcome.needs_review, "amount > $10000 must trigger review");
}

#[test]
fn se_03_negative_amount_with_rent_signal_lower_confidence() {
    // Negative rent-signal transaction → RentalIncome at 0.80, review: true.
    let sample = SampleTransaction {
        tx_id: "se-03".into(),
        account_id: "WF--BH-CHK--2024-03".into(),
        date: "2024-03-10".into(),
        amount: "-500.00".into(),
        description: "Rental property repair — plumber".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_schedule_e.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "RentalIncome");
    assert!(
        (outcome.confidence - 0.80).abs() < f64::EPSILON,
        "expected 0.80, got {}",
        outcome.confidence
    );
    assert!(
        outcome.needs_review,
        "negative amount with rent signal must trigger review"
    );
}

// ---------------------------------------------------------------------------
// classify_fbar.rhai — 31 USC §5314
// ---------------------------------------------------------------------------

#[test]
fn fbar_01_hsbc_account_classifies_foreign_income() {
    // HSBC in account_id is a foreign account signal.
    let sample = SampleTransaction {
        tx_id: "fbar-01".into(),
        account_id: "HSBC--BH-SAV--2024-01".into(),
        date: "2024-01-15".into(),
        amount: "5000.00".into(),
        description: "Monthly interest credit".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_fbar.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "ForeignIncome");
    assert!(
        (outcome.confidence - 0.82).abs() < f64::EPSILON,
        "expected 0.82, got {}",
        outcome.confidence
    );
    assert!(
        !outcome.needs_review,
        "amount 5000 below $9000 FBAR near-threshold trigger"
    );
}

#[test]
fn fbar_02_near_threshold_triggers_review() {
    // abs(amount) > $9000 → review: true (approaching $10,000 FBAR threshold).
    let sample = SampleTransaction {
        tx_id: "fbar-02".into(),
        account_id: "HSBC--BH-SAV--2024-06".into(),
        date: "2024-06-30".into(),
        amount: "9500.00".into(),
        description: "Wire transfer from HSBC account".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_fbar.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "ForeignIncome");
    assert!(
        outcome.needs_review,
        "amount > $9000 must trigger FBAR review"
    );
    assert!(
        outcome.reason.contains("31 USC §5314"),
        "reason must cite 31 USC §5314; got: {}",
        outcome.reason
    );
}

#[test]
fn fbar_03_sepa_description_classifies_foreign_income() {
    // "SEPA" in description (no special account_id).
    let sample = SampleTransaction {
        tx_id: "fbar-03".into(),
        account_id: "WF--BH-CHK--2024-02".into(),
        date: "2024-02-20".into(),
        amount: "3000.00".into(),
        description: "SEPA payment received from EU supplier".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_fbar.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "ForeignIncome");
    assert!(!outcome.needs_review, "3000 is below $9000 near-threshold");
}

#[test]
fn fbar_04_no_signal_unclassified() {
    let sample = SampleTransaction {
        tx_id: "fbar-04".into(),
        account_id: "WF--BH-CHK--2024-01".into(),
        date: "2024-01-05".into(),
        amount: "100.00".into(),
        description: "Coffee shop purchase".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_fbar.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "Unclassified");
}

// ---------------------------------------------------------------------------
// classify_fatca.rhai — IRC §6038D
// ---------------------------------------------------------------------------

#[test]
fn fatca_01_high_value_foreign_financial_triggers_review() {
    // abs(amount) > 25000 with HSBC account → ForeignIncome, review: true.
    let sample = SampleTransaction {
        tx_id: "fatca-01".into(),
        account_id: "HSBC--BH-INV--2024-09".into(),
        date: "2024-09-01".into(),
        amount: "50000.00".into(),
        description: "Foreign investment distribution".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_fatca.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "ForeignIncome");
    assert!(
        outcome.needs_review,
        "amount > 25000 must trigger FATCA review"
    );
    assert!(
        outcome.reason.contains("IRC §6038D"),
        "reason must cite IRC §6038D; got: {}",
        outcome.reason
    );
}

#[test]
fn fatca_02_low_value_no_review() {
    // Same signal but amount < 25000 → confidence 0.65, review: false.
    let sample = SampleTransaction {
        tx_id: "fatca-02".into(),
        account_id: "HSBC--BH-SAV--2024-04".into(),
        date: "2024-04-10".into(),
        amount: "10000.00".into(),
        description: "Offshore savings interest".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_fatca.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "ForeignIncome");
    assert!(
        (outcome.confidence - 0.65).abs() < f64::EPSILON,
        "expected 0.65 for low-value FATCA signal, got {}",
        outcome.confidence
    );
    assert!(
        !outcome.needs_review,
        "amount 10000 below 25000 → no review required"
    );
}

#[test]
fn fatca_03_no_signal_unclassified() {
    let sample = SampleTransaction {
        tx_id: "fatca-03".into(),
        account_id: "WF--BH-CHK--2024-01".into(),
        date: "2024-01-10".into(),
        amount: "500.00".into(),
        description: "Grocery run".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_fatca.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "Unclassified");
}

// ---------------------------------------------------------------------------
// classify_crypto_trading.rhai — IRS Notice 2014-21; Rev. Proc. 2024-28
// ---------------------------------------------------------------------------

#[test]
fn ct_01_crypto_buy_classifies_transfer() {
    // BUY signal → Transfer (basis establishment, not taxable), review: false.
    let sample = SampleTransaction {
        tx_id: "ct-01".into(),
        account_id: "COINBASE--BH-CRYPTO--2024-01".into(),
        date: "2024-01-20".into(),
        amount: "-5000.00".into(),
        description: "BTC buy — 0.08 BTC at 62500 USD".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_crypto_trading.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "Transfer");
    assert!(
        (outcome.confidence - 0.90).abs() < f64::EPSILON,
        "expected 0.90, got {}",
        outcome.confidence
    );
    assert!(
        !outcome.needs_review,
        "buy is not a taxable event — no review needed"
    );
}

#[test]
fn ct_02_crypto_sell_positive_classifies_crypto_gain() {
    // SELL with positive proceeds → CryptoGain, review: true.
    let sample = SampleTransaction {
        tx_id: "ct-02".into(),
        account_id: "COINBASE--BH-CRYPTO--2024-06".into(),
        date: "2024-06-15".into(),
        amount: "7200.00".into(),
        description: "BTC sell — 0.08 BTC at 90000 USD".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_crypto_trading.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "CryptoGain");
    assert!(
        (outcome.confidence - 0.85).abs() < f64::EPSILON,
        "expected 0.85, got {}",
        outcome.confidence
    );
    assert!(
        outcome.needs_review,
        "crypto sale is a capital event — must review"
    );
}

#[test]
fn ct_03_crypto_sell_negative_classifies_capital_loss() {
    // SELL with negative net → CapitalLoss, review: true.
    let sample = SampleTransaction {
        tx_id: "ct-03".into(),
        account_id: "COINBASE--BH-CRYPTO--2024-09".into(),
        date: "2024-09-10".into(),
        amount: "-400.00".into(),
        description: "ETH sell — realized loss on position".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_crypto_trading.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "CapitalLoss");
    assert!(
        outcome.needs_review,
        "crypto loss is a capital event — must review"
    );
}

#[test]
fn ct_04_exchange_deposit_is_transfer() {
    let sample = SampleTransaction {
        tx_id: "ct-04".into(),
        account_id: "COINBASE--BH-CRYPTO--2024-02".into(),
        date: "2024-02-01".into(),
        amount: "2000.00".into(),
        description: "Exchange deposit — USD funding for coin purchase".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_crypto_trading.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "Transfer");
    assert!(!outcome.needs_review);
}

#[test]
fn ct_05_no_signal_unclassified() {
    let sample = SampleTransaction {
        tx_id: "ct-05".into(),
        account_id: "WF--BH-CHK--2024-01".into(),
        date: "2024-01-05".into(),
        amount: "50.00".into(),
        description: "Coffee and lunch".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_crypto_trading.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "Unclassified");
}

// ---------------------------------------------------------------------------
// classify_crypto_staking.rhai — Rev. Rul. 2023-14
// ---------------------------------------------------------------------------

#[test]
fn cs_01_staking_reward_classifies_crypto_income() {
    let sample = SampleTransaction {
        tx_id: "cs-01".into(),
        account_id: "COINBASE--BH-CRYPTO--2024-03".into(),
        date: "2024-03-15".into(),
        amount: "125.00".into(),
        description: "Staking reward — ETH validator node".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_crypto_staking.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "CryptoIncome");
    assert!(
        (outcome.confidence - 0.88).abs() < f64::EPSILON,
        "expected 0.88, got {}",
        outcome.confidence
    );
    assert!(
        outcome.needs_review,
        "staking reward always requires FMV review"
    );
    assert!(
        outcome.reason.contains("Rev. Rul. 2023-14"),
        "reason must cite Rev. Rul. 2023-14; got: {}",
        outcome.reason
    );
}

#[test]
fn cs_02_airdrop_lower_confidence() {
    // "airdrop" alone → CryptoIncome at 0.80 (valuation uncertainty).
    let sample = SampleTransaction {
        tx_id: "cs-02".into(),
        account_id: "COINBASE--BH-CRYPTO--2024-05".into(),
        date: "2024-05-01".into(),
        amount: "200.00".into(),
        description: "Airdrop received — new token distribution".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_crypto_staking.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "CryptoIncome");
    assert!(
        (outcome.confidence - 0.80).abs() < f64::EPSILON,
        "expected 0.80 for airdrop, got {}",
        outcome.confidence
    );
    assert!(outcome.needs_review, "airdrop must always trigger review");
}

#[test]
fn cs_03_defi_yield_classifies_crypto_income() {
    let sample = SampleTransaction {
        tx_id: "cs-03".into(),
        account_id: "DEFI--BH-WALLET--2024-04".into(),
        date: "2024-04-10".into(),
        amount: "55.00".into(),
        description: "DeFi yield — liquidity mining reward".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_crypto_staking.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "CryptoIncome");
    assert!(outcome.needs_review);
}

#[test]
fn cs_04_no_signal_unclassified() {
    let sample = SampleTransaction {
        tx_id: "cs-04".into(),
        account_id: "WF--BH-CHK--2024-01".into(),
        date: "2024-01-10".into(),
        amount: "100.00".into(),
        description: "Gym membership".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_crypto_staking.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "Unclassified");
}

// ---------------------------------------------------------------------------
// classify_au_gst.rhai — A New Tax System (GST) Act 1999 s.11-5
// ---------------------------------------------------------------------------

#[test]
fn ag_01_negative_gst_expense_classifies_au_gst() {
    // Negative amount with GST signal → AuGst (creditable acquisition).
    let sample = SampleTransaction {
        tx_id: "ag-01".into(),
        account_id: "ANZ--BH-CHK--2024-02".into(),
        date: "2024-02-14".into(),
        amount: "-440.00".into(),
        description: "Tax invoice — office supplies inc. GST".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_au_gst.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "AuGst");
    assert!(
        (outcome.confidence - 0.85).abs() < f64::EPSILON,
        "expected 0.85, got {}",
        outcome.confidence
    );
    assert!(!outcome.needs_review);
}

#[test]
fn ag_02_positive_au_income_classifies_foreign_income_with_review() {
    // Positive amount with AU account → ForeignIncome, review: true.
    let sample = SampleTransaction {
        tx_id: "ag-02".into(),
        account_id: "CBA--BH-BIZ--2024-03".into(),
        date: "2024-03-01".into(),
        amount: "5500.00".into(),
        description: "Client payment with ABN quoted — services rendered".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_au_gst.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "ForeignIncome");
    assert!(
        (outcome.confidence - 0.78).abs() < f64::EPSILON,
        "expected 0.78, got {}",
        outcome.confidence
    );
    assert!(
        outcome.needs_review,
        "AU income for US expat always requires review"
    );
}

#[test]
fn ag_03_no_signal_unclassified() {
    let sample = SampleTransaction {
        tx_id: "ag-03".into(),
        account_id: "WF--BH-CHK--2024-01".into(),
        date: "2024-01-20".into(),
        amount: "300.00".into(),
        description: "Online purchase — electronics".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_au_gst.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "Unclassified");
}

// ---------------------------------------------------------------------------
// classify_au_cgt.rhai — ITAA 1997 s.115-A
// ---------------------------------------------------------------------------

#[test]
fn ac_01_au_property_sale_classifies_au_cgt() {
    // "property sale" + AU account → AuCgt, confidence 0.87, review: true.
    let sample = SampleTransaction {
        tx_id: "ac-01".into(),
        account_id: "ANZ--AU-PROP--2024-10".into(),
        date: "2024-10-01".into(),
        amount: "650000.00".into(),
        description: "Property sale settlement — Melbourne investment property".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_au_cgt.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "AuCgt");
    assert!(
        (outcome.confidence - 0.87).abs() < f64::EPSILON,
        "expected 0.87, got {}",
        outcome.confidence
    );
    assert!(
        outcome.needs_review,
        "AU CGT always requires review for discount eligibility"
    );
    assert!(
        outcome.reason.contains("ITAA 1997 s.115-A"),
        "reason must cite ITAA 1997 s.115-A; got: {}",
        outcome.reason
    );
}

#[test]
fn ac_02_shares_sale_with_aud_signal_classifies_au_cgt() {
    // "shares sale" + "AUD" in description (no special account_id needed).
    let sample = SampleTransaction {
        tx_id: "ac-02".into(),
        account_id: "COMMSEC--AU-BROK--2024-08".into(),
        date: "2024-08-15".into(),
        amount: "12500.00".into(),
        description: "Shares sale — ASX-listed stock capital proceeds AUD".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_au_cgt.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "AuCgt");
    assert!(outcome.needs_review);
}

#[test]
fn ac_03_capital_signal_without_au_jurisdiction_unclassified() {
    // Capital event keyword present but no AU jurisdiction indicator → Unclassified.
    let sample = SampleTransaction {
        tx_id: "ac-03".into(),
        account_id: "FIDELITY--US-BROK--2024-06".into(),
        date: "2024-06-10".into(),
        amount: "8000.00".into(),
        description: "Capital proceeds from property sale — US domestic".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_au_cgt.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(
        outcome.category, "Unclassified",
        "US domestic capital event must not match AU CGT rule"
    );
}

#[test]
fn ac_04_no_signal_unclassified() {
    let sample = SampleTransaction {
        tx_id: "ac-04".into(),
        account_id: "WF--BH-CHK--2024-01".into(),
        date: "2024-01-05".into(),
        amount: "50.00".into(),
        description: "Fuel — petrol station".into(),
    };
    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_au_cgt.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule failed: {e}"));

    assert_eq!(outcome.category, "Unclassified");
}
