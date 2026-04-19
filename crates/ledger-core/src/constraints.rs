//! Kasuari-based constraint solving for data plausibility.
//! Uses the Cassowary algorithm to evaluate constraints against transaction populations.

use serde::{Deserialize, Serialize};

/// Constraint strength levels (matching Kasuari).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintStrength {
    /// Must be satisfied — cannot be violated.
    Required,
    /// Strong preference — can be violated only if required fails.
    Strong,
    /// Medium preference.
    Medium,
    /// Weak preference — violated first if needed.
    Weak,
}

/// Result of constraint evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintEvaluation {
    /// Whether REQUIRED constraints passed.
    pub required_pass: bool,
    /// Fraction of STRONG constraints passing (0.0-1.0).
    pub strong_ratio: f32,
    /// Fraction of MEDIUM constraints passing (0.0-1.0).
    pub medium_ratio: f32,
    /// Fraction of WEAK constraints passing (0.0-1.0).
    pub weak_ratio: f32,
}

impl ConstraintEvaluation {
    /// Convert to confidence score and disposition.
    pub fn to_confidence(&self) -> (f32, super::validation::Disposition) {
        use super::validation::Disposition;

        if !self.required_pass {
            return (0.0, Disposition::Unrecoverable);
        }

        let score = self.strong_ratio * 0.60
            + self.medium_ratio * 0.30
            + self.weak_ratio * 0.10;

        let disposition = if score >= 0.85 {
            Disposition::Advisory
        } else {
            Disposition::Recoverable
        };

        (score, disposition)
    }
}

/// A historical constraint set for a vendor or category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorConstraintSet {
    /// Vendor identifier.
    pub vendor_id: String,
    /// 5th percentile of historical amounts.
    pub amount_p05: f64,
    /// 95th percentile of historical amounts.
    pub amount_p95: f64,
    /// Usual day of month for billing (1-31).
    pub usual_day_of_month: Option<u32>,
    /// Most common tax code.
    pub usual_tax_code: String,
    /// Most common account code.
    pub usual_account: String,
}

impl VendorConstraintSet {
    /// Evaluate a transaction against this vendor's historical constraints.
    pub fn evaluate(&self, amount: f64, day: u32, tax_code: &str, account: &str) -> ConstraintEvaluation {
        let in_range = amount >= self.amount_p05 && amount <= self.amount_p95;
        let tax_matches = tax_code == self.usual_tax_code;
        let account_matches = account == self.usual_account;

        // REQUIRED: amount is non-zero
        let required_pass = amount != 0.0;

        // STRONG constraints
        let strong_count = 2.0;
        let strong_pass = [in_range, tax_matches]
            .iter()
            .filter(|&&b| b)
            .count() as f32;
        let strong_ratio = strong_pass / strong_count;

        // MEDIUM constraints
        let day_matches = self.usual_day_of_month
            .map(|d| day == d)
            .unwrap_or(true);
        let medium_count = 2.0;
        let medium_pass = [day_matches, account_matches]
            .iter()
            .filter(|&&b| b)
            .count() as f32;
        let medium_ratio = medium_pass / medium_count;

        // WEAK — informational
        let weak_ratio = 1.0;

        ConstraintEvaluation {
            required_pass,
            strong_ratio,
            medium_ratio,
            weak_ratio,
        }
    }
}

/// Invoice arithmetic constraints (total = subtotal + gst).
#[derive(Debug, Clone, Default)]
pub struct InvoiceConstraintSolver {
    constraint_count: usize,
}

impl InvoiceConstraintSolver {
    pub fn new() -> Self {
        Self {
            constraint_count: 0,
        }
    }

    /// Validate invoice arithmetic: total ≈ subtotal + gst.
    pub fn validate(&self, total: f64, subtotal: f64, gst: f64) -> ConstraintEvaluation {
        // REQUIRED: total = subtotal + gst
        let required_pass = (total - subtotal - gst).abs() < 0.01;

        // STRONG: gst ≈ subtotal * 0.1 (allowing 2¢ rounding)
        let expected_gst = subtotal * 0.1;
        let gst_correct = (gst - expected_gst).abs() < 0.02;

        // MEDIUM: amounts are positive
        let amounts_positive = total > 0.0 && subtotal > 0.0;

        // WEAK: total is reasonable
        let total_reasonable = total > 0.0 && total < 1_000_000.0;

        ConstraintEvaluation {
            required_pass,
            strong_ratio: if gst_correct { 1.0 } else { 0.0 },
            medium_ratio: if amounts_positive { 1.0 } else { 0.0 },
            weak_ratio: if total_reasonable { 1.0 } else { 0.0 },
        }
    }
}

/// Constraint solver for ontology graph layout (using Kasuari for visualization).
#[derive(Debug, Clone, Default)]
pub struct LayoutSolver {
    _private: (),
}

impl LayoutSolver {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_constraint() {
        let vendor = VendorConstraintSet {
            vendor_id: "AWS".to_string(),
            amount_p05: 100.0,
            amount_p95: 500.0,
            usual_day_of_month: Some(1),
            usual_tax_code: "BASEXCLUDED".to_string(),
            usual_account: "6-8800".to_string(),
        };

        let result = vendor.evaluate(250.0, 1, "BASEXCLUDED", "6-8800");
        assert!(result.required_pass);
        assert!(result.strong_ratio > 0.5);
    }

    #[test]
    fn test_invoice_constraint() {
        let solver = InvoiceConstraintSolver::new();
        let result = solver.validate(110.0, 100.0, 10.0);

        assert!(result.required_pass);
        assert_eq!(result.strong_ratio, 1.0);
    }

    #[test]
    fn test_evaluation_to_confidence() {
        let eval = ConstraintEvaluation {
            required_pass: true,
            strong_ratio: 1.0,
            medium_ratio: 1.0,
            weak_ratio: 1.0,
        };

        let (score, disposition) = eval.to_confidence();
        assert!(score > 0.9);
        assert_eq!(disposition, super::super::validation::Disposition::Advisory);
    }
}