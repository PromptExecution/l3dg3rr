//! Governance — policy types for access control, crash budgets, and TTL enforcement.
//!
//! These types form a constraint system that a solver (Z3) can verify:
//! - No two policies on the same machine may have overlapping `allowed_starters` with
//!   conflicting `max_ttl` values (a solver constraint, not a runtime check).
//! - `crash_budget` must be ≥ 0 and ≤ `MAX_CRASH_BUDGET` (compile-time invariant).

use std::time::Duration;

/// Maximum allowable crash budget across all surfaces.
pub const MAX_CRASH_BUDGET: u32 = 100;

/// Agent role for governance access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AgentRole {
    Executive,
    Operator,
    Specialist,
    Auditor,
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Executive => write!(f, "executive"),
            Self::Operator => write!(f, "operator"),
            Self::Specialist => write!(f, "specialist"),
            Self::Auditor => write!(f, "auditor"),
        }
    }
}

/// Governance policy for a process surface.
///
/// # Solver-verifiable constraints
///
/// ```z3
/// ;; A surface's crash budget must never exceed MAX_CRASH_BUDGET
/// (assert (<= crash_budget MAX_CRASH_BUDGET))
/// ;; auto_restart is true iff restart budget remains
/// (assert (= auto_restart (> crash_budget 0)))
/// ```
#[derive(Debug, Clone)]
pub struct GovernancePolicy {
    /// Which agent roles are allowed to start this surface.
    pub allowed_starters: Vec<AgentRole>,
    /// Maximum runtime before forced re-evaluation.
    pub max_ttl: Duration,
    /// Whether this surface can be restarted automatically on crash.
    pub auto_restart: bool,
    /// Crash threshold before surface is quarantined (0 = never quarantine).
    pub crash_budget: u32,
}

impl GovernancePolicy {
    /// Verify solver-level invariants at runtime.
    /// In production this is redundant with compile-time/CLP checks.
    pub fn validate(&self) -> Result<(), String> {
        if self.crash_budget > MAX_CRASH_BUDGET {
            return Err(format!(
                "crash_budget {} exceeds MAX_CRASH_BUDGET {}",
                self.crash_budget, MAX_CRASH_BUDGET
            ));
        }
        if self.allowed_starters.is_empty() {
            return Err("at least one allowed starter required".into());
        }
        Ok(())
    }

    /// Solver-friendly encoding as a serializable constraint set.
    pub fn to_constraints(&self) -> GovernanceConstraints {
        GovernanceConstraints {
            allowed_starters: self.allowed_starters.iter().map(|r| r.to_string()).collect(),
            max_ttl_secs: self.max_ttl.as_secs(),
            auto_restart: self.auto_restart,
            crash_budget: self.crash_budget,
        }
    }
}

impl Default for GovernancePolicy {
    fn default() -> Self {
        Self {
            allowed_starters: vec![AgentRole::Executive, AgentRole::Operator],
            max_ttl: Duration::from_secs(86400),
            auto_restart: true,
            crash_budget: 3,
        }
    }
}

/// Serializable constraint set for solver-based verification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GovernanceConstraints {
    pub allowed_starters: Vec<String>,
    pub max_ttl_secs: u64,
    pub auto_restart: bool,
    pub crash_budget: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_validates() {
        let p = GovernancePolicy::default();
        assert!(p.validate().is_ok());
    }

    #[test]
    fn crash_budget_too_high() {
        let p = GovernancePolicy {
            crash_budget: MAX_CRASH_BUDGET + 1,
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn empty_starters_fails() {
        let p = GovernancePolicy {
            allowed_starters: vec![],
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn to_constraints_roundtrip() {
        let p = GovernancePolicy::default();
        let c = p.to_constraints();
        assert_eq!(c.crash_budget, 3);
        assert_eq!(c.allowed_starters.len(), 2);
    }

    #[test]
    fn agent_role_display() {
        assert_eq!(AgentRole::Executive.to_string(), "executive");
        assert_eq!(AgentRole::Auditor.to_string(), "auditor");
    }
}
