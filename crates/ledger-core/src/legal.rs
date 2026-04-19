//! Z3-based legal rule verification for tax compliance.
//! Encodes tax rules as logical formulas and transaction facts as assertions.
//! 
//! Note: Z3 API uses a functional style. This module provides a simplified interface
//! for legal rule verification that abstracts the Z3 complexity.

use serde::{Deserialize, Serialize};

/// Jurisdiction for tax rule evaluation (US, AU, UK, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Jurisdiction {
    US,
    AU,
    UK,
}

impl Default for Jurisdiction {
    fn default() -> Self {
        Self::US
    }
}

impl Jurisdiction {
    pub fn code(&self) -> &'static str {
        match self {
            Self::US => "US",
            Self::AU => "AU",
            Self::UK => "UK",
        }
    }
}

/// Result of Z3 SAT check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Z3Result {
    /// Rule is satisfied.
    Satisfied,
    /// Rule is violated — witness shows which condition failed.
    Violated { witness: String },
    /// Solver could not determine.
    Unknown,
}

/// A legal rule encoded for verification.
/// The formula describes logical conditions that must hold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalRule {
    /// Unique identifier.
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// Which jurisdiction this rule belongs to.
    pub jurisdiction: Jurisdiction,
    /// Human-readable formula description.
    pub formula: String,
    /// Category (GST, income, deduction, FBAR, etc.).
    pub category: String,
}

impl LegalRule {
    /// Create a new legal rule.
    pub fn new(id: impl Into<String>, jurisdiction: Jurisdiction) -> Self {
        Self {
            id: id.into(),
            description: String::new(),
            jurisdiction,
            formula: String::new(),
            category: String::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_formula(mut self, formula: impl Into<String>) -> Self {
        self.formula = formula.into();
        self
    }

    pub fn with_category(mut self, cat: impl Into<String>) -> Self {
        self.category = cat.into();
        self
    }
}

/// Transaction facts for rule evaluation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransactionFacts {
    /// Vendor jurisdiction code.
    pub vendor_jurisdiction: Option<String>,
    /// Supply type (SaaS, service, goods).
    pub supply_type: Option<String>,
    /// Tax code applied.
    pub tax_code: Option<String>,
    /// Amount in local currency.
    pub amount: Option<String>,
    /// Is business activity.
    pub is_business_activity: Option<bool>,
    /// Is ordinary expense.
    pub is_ordinary: Option<bool>,
    /// Is necessary expense.
    pub is_necessary: Option<bool>,
}

impl TransactionFacts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_vendor(mut self, j: impl Into<String>) -> Self {
        self.vendor_jurisdiction = Some(j.into());
        self
    }

    pub fn with_supply_type(mut self, t: impl Into<String>) -> Self {
        self.supply_type = Some(t.into());
        self
    }

    pub fn with_tax_code(mut self, c: impl Into<String>) -> Self {
        self.tax_code = Some(c.into());
        self
    }

    pub fn with_amount(mut self, a: impl Into<String>) -> Self {
        self.amount = Some(a.into());
        self
    }
}

/// Simplified legal verification (mock for demonstration).
/// Real implementation would use Z3, but the interface is defined here.
pub struct LegalSolver;

impl Default for LegalSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl LegalSolver {
    pub fn new() -> Self {
        Self
    }

    /// Verify transaction against a legal rule.
    /// Returns whether the facts satisfy the rule conditions.
    pub fn verify(&self, rule: &LegalRule, facts: &TransactionFacts) -> Z3Result {
        // Simplified mock: check AU GST rules
        if rule.id.contains("au-gst-38-190") {
            // Rule: overseas SaaS → BASEXCLUDED
            if facts.vendor_jurisdiction.as_deref() == Some("US")
                || facts.vendor_jurisdiction.as_deref() == Some("UK")
            {
                if facts.supply_type.as_deref() == Some("SaaS") {
                    // Should have BASEXCLUDED tax code
                    if facts.tax_code.as_deref() == Some("BASEXCLUDED") {
                        return Z3Result::Satisfied;
                    } else {
                        return Z3Result::Violated {
                            witness: "US/UK SaaS should have BASEXCLUDED tax code".to_string(),
                        };
                    }
                }
            }
            // AU vendors with SaaS should have INPUT
            if facts.vendor_jurisdiction.as_deref() == Some("AU")
                && facts.supply_type.as_deref() == Some("SaaS")
            {
                if facts.tax_code.as_deref() == Some("INPUT") {
                    return Z3Result::Satisfied;
                } else {
                    return Z3Result::Violated {
                        witness: "AU SaaS should have INPUT tax code".to_string(),
                    };
                }
            }
        }

        // US Schedule C rules
        if rule.id.contains("schedule-c") {
            let is_business = facts.is_business_activity.unwrap_or(false);
            let is_ord = facts.is_ordinary.unwrap_or(false);
            let is_nec = facts.is_necessary.unwrap_or(false);

            if is_business && is_ord && is_nec {
                return Z3Result::Satisfied;
            } else if is_business {
                return Z3Result::Violated {
                    witness: "must be ordinary AND necessary".to_string(),
                };
            }
        }

        // Default: don't know
        Z3Result::Unknown
    }
}

/// Example: AU GST Act s38-190 — overseas SaaS is input-taxed supply.
pub mod au_gst {
    use super::*;

    /// Rule: IF vendor.jurisdiction != AU AND supply.type == SaaS THEN tax_code == BASEXCLUDED
    pub fn rule_38_190() -> LegalRule {
        LegalRule::new("au-gst-38-190", Jurisdiction::AU)
            .with_description("Overseas SaaS is input-taxed supply under GST Act s38-190")
            .with_category("GST")
            .with_formula("vendor != AU AND type == SaaS → tax_code == BASEXCLUDED")
    }
}

/// Example: US Schedule C deduction rules.
pub mod us_schedule_c {
    use super::*;

    /// Rule: Business expenses are deductible if ordinary and necessary.
    pub fn rule_ordinary_necessary() -> LegalRule {
        LegalRule::new("us-schedule-c-ordinary-necessary", Jurisdiction::US)
            .with_description("Expenses must be ordinary and necessary for business")
            .with_category("deduction")
            .with_formula("business_activity AND ordinary AND necessary → deductible")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_au_gst_us_saas() {
        let solver = LegalSolver::new();
        let rule = au_gst::rule_38_190();
        let facts = TransactionFacts::new()
            .with_vendor("US")
            .with_supply_type("SaaS")
            .with_tax_code("BASEXCLUDED");

        let result = solver.verify(&rule, &facts);
        assert_eq!(result, Z3Result::Satisfied);
    }

    #[test]
    fn test_au_gst_us_wrong_tax() {
        let solver = LegalSolver::new();
        let rule = au_gst::rule_38_190();
        let facts = TransactionFacts::new()
            .with_vendor("US")
            .with_supply_type("SaaS")
            .with_tax_code("INPUT"); // wrong for US SaaS

        let result = solver.verify(&rule, &facts);
        assert!(matches!(result, Z3Result::Violated { .. }));
    }

    #[test]
    fn test_us_schedule_c() {
        let solver = LegalSolver::new();
        let rule = us_schedule_c::rule_ordinary_necessary();
        let mut facts = TransactionFacts::new();
        facts.is_business_activity = Some(true);
        facts.is_ordinary = Some(true);
        facts.is_necessary = Some(true);

        let result = solver.verify(&rule, &facts);
        assert_eq!(result, Z3Result::Satisfied);
    }
}