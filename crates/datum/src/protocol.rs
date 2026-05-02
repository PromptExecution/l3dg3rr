//! Protocol encoding theory — Z3/Kasuari linear constraints for datum protocol optimality.
//!
//! This module encodes the protocol optimization rules as a constraint system.
//! It does NOT call the Z3 or Kasuari solvers at runtime — instead it defines
//! the constraint vocabulary and evaluates them against concrete protocol choices.
//!
//! ## Core question
//!
//! When a datum or MCP provider exports a protocol, should a directory of the same
//! name store binary artifacts alongside the text contract?
//!
//! ## Constraint vocabulary
//!
//! | Symbol | Meaning | Domain |
//! |--------|---------|--------|
//! | `P`    | Has a text protocol contract (TOML, JSON, .tomllmd) | {0,1} |
//! | `B`    | Has a sibling binary store directory of the same name | {0,1} |
//! | `S`    | Protocol encoding is self-contained (no external deps) | {0,1} |
//! | `T`    | Protocol supports type-safe serialization (rkyv, protobuf) | {0,1} |
//! | `M`    | Protocol supports multiple summary levels (verbatim/exec/epigram) | {0,1} |
//! | `C`    | Protocol uses content-hash addressing (blake3) | {0,1} |
//! | `E`    | Protocol declares entanglement edges to other datums | {0,1} |
//! | `O`    | Protocol is optimal (derived goal predicate) | {0,1} |
//!
//! ## Z3-style constraints (linear integer arithmetic, pseudo-smtlib)
//!
//! ```text
//! ; A protocol is optimal IFF it has a text contract AND
//! ; exactly one of (self-contained, binary store) is true.
//! ; DEF: xor(a,b) = (a != b) in integer arithmetic
//! (define-fun xor ((a Bool) (b Bool)) Bool (or (and a (not b)) (and b (not a))))
//! (assert (= O (and P (xor S B))))
//!
//! ; Self-contained protocol requires type-safe serialization
//! (assert (=> S T))
//!
//! ; Binary store requires content-hash addressing for dedup
//! (assert (=> B C))
//!
//! ; No protocol form can have both P and B true (no hybrid redundancy)
//! (assert (not (and P B)))
//!
//! ; Multi-level protocols need entanglement edges
//! (assert (=> M E))
//!
//! ; Content hash plus binary store implies type safety
//! (assert (=> (and C B) T))
//!
//! ; Protocols with entanglement are never purely self-contained
//! (assert (=> E (not S)))
//! ```
//!
//! ## Kasuari-style constraint strengths
//!
//! Kasuari constraints are soft plausibility rules with strengths:
//! - STRONG (Required): Violation produces an Issue with Disposition::Unrecoverable
//! - MEDIUM (Expected): Violation produces a Warning
//! - WEAK (Suggested): Violation produces an Info note
//!
//! ```text
//! ; STRONG: Every protocol must have a text contract XOR binary store
//! (P xor B)  [strength: STRONG]
//!
//! ; STRONG: No hybrid (P and B is forbidden)
//! (not (and P B))  [strength: STRONG]
//!
//! ; MEDIUM: If binary store exists, use content hashes
//! B → C  [strength: MEDIUM]
//!
//! ; MEDIUM: Self-contained protocols should use type-safe ser
//! S → T  [strength: MEDIUM]
//!
//! ; WEAK: Multi-level protocols benefit from entanglement
//! M → E  [strength: WEAK]
//!
//! ; WEAK: Prefer text contract when both are possible
//! P → (not B)  [strength: WEAK]
//! ```
//!
//! ## Dialect comparison: where each choice leads
//!
//! | Protocol style | P | B | S | T | M | C | E | O | Likely scenario |
//! |---|---|---|---|---|---|---|---|---|-----------------|
//! | Pure text .tomllmd | 1 | 0 | 1 | 1 | 1 | 0 | 1 | **1** | b00t datum, self-describing |
//! | Binary + content hash | 0 | 1 | 0 | 1 | 0 | 1 | 0 | **1** | rkyv archive, fast recall |
//! | Hybrid (text + binary dir) | 1 | 1 | 0 | 1 | 1 | 1 | 1 | **0** | Redundant — P+B forbidden |
//! | No protocol | 0 | 0 | 0 | 0 | 0 | 0 | 0 | **0** | Ad-hoc, not recommended |
//! | Text only, no types | 1 | 0 | 1 | 0 | 0 | 0 | 0 | **0** | Fragile, no validation |
//! | Binary with text stub | 1 | 1 | 0 | 1 | 0 | 1 | 0 | **0** | P+B forbidden |

use crate::ast::LintSeverity;

/// Protocol constraint strength (Kasuari-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintStrength {
    Strong,
    Medium,
    Weak,
}

/// A single protocol constraint evaluation.
#[derive(Debug, Clone)]
pub struct ProtocolConstraint {
    pub name: &'static str,
    pub strength: ConstraintStrength,
    pub passed: bool,
    pub message: String,
}

/// Boolean features of a protocol encoding.
#[derive(Debug, Clone, Default)]
pub struct ProtocolFeatures {
    pub has_text_contract: bool,
    pub has_binary_store: bool,
    pub is_self_contained: bool,
    pub has_type_safety: bool,
    pub has_multi_level: bool,
    pub has_content_hash: bool,
    pub has_entanglement: bool,
}

/// Evaluate protocol optimality using the Z3-style linear constraints.
/// Returns both the derived O (optimal) predicate and which constraints fired.
pub fn evaluate_protocol(features: &ProtocolFeatures) -> (bool, Vec<ProtocolConstraint>) {
    let mut constraints = Vec::new();

    // STRONG: P XOR B (exactly one of text contract or binary store)
    let has_any = features.has_text_contract || features.has_binary_store;
    let has_both = features.has_text_contract && features.has_binary_store;
    let xor_ok = has_any && !has_both;
    constraints.push(ProtocolConstraint {
        name: "P XOR B",
        strength: ConstraintStrength::Strong,
        passed: xor_ok,
        message: if xor_ok && features.has_text_contract {
            "Text contract without binary store — clean".into()
        } else if xor_ok && features.has_binary_store {
            "Binary store without text contract — clean".into()
        } else if has_both {
            "Redundant: both text contract and binary store active".into()
        } else {
            "Neither text contract nor binary store present".into()
        },
    });

    // STRONG: not(P and B)  (no hybrid redundancy)
    let no_hybrid_ok = !has_both;
    constraints.push(ProtocolConstraint {
        name: "not(P and B)",
        strength: ConstraintStrength::Strong,
        passed: no_hybrid_ok,
        message: if no_hybrid_ok {
            "No hybrid redundancy".into()
        } else {
            "Hybrid text+binary protocol — violates separation".into()
        },
    });

    // O = P XOR B (exactly one of text contract or binary store)
    let p = features.has_text_contract;
    let b = features.has_binary_store;
    let optimal = p != b;

    // MEDIUM: B → C  (binary store requires content hashes)
    if features.has_binary_store {
        let bc_ok = features.has_content_hash;
        constraints.push(ProtocolConstraint {
            name: "B → C",
            strength: ConstraintStrength::Medium,
            passed: bc_ok,
            message: if bc_ok {
                "Binary store uses content-hash addressing".into()
            } else {
                "Binary store without content-hash — dedup risk".into()
            },
        });
    }

    // MEDIUM: S → T  (self-contained requires type safety)
    if features.is_self_contained {
        let st_ok = features.has_type_safety;
        constraints.push(ProtocolConstraint {
            name: "S → T",
            strength: ConstraintStrength::Medium,
            passed: st_ok,
            message: if st_ok {
                "Self-contained protocol uses type-safe serialization".into()
            } else {
                "Self-contained protocol lacks type safety".into()
            },
        });
    }

    // WEAK: M → E  (multi-level benefits from entanglement)
    if features.has_multi_level {
        let me_ok = features.has_entanglement;
        constraints.push(ProtocolConstraint {
            name: "M → E",
            strength: ConstraintStrength::Weak,
            passed: me_ok,
            message: if me_ok {
                "Multi-level protocol has entanglement edges".into()
            } else {
                "Multi-level protocol without entanglement — lost cross-refs".into()
            },
        });
    }

    (optimal, constraints)
}

/// Check if any STRONG constraint failed (unrecoverable disposition).
pub fn has_unrecoverable_violations(constraints: &[ProtocolConstraint]) -> bool {
    constraints
        .iter()
        .any(|c| c.strength == ConstraintStrength::Strong && !c.passed)
}

/// Return constraint violations grouped by severity.
pub fn classify_violations(
    constraints: &[ProtocolConstraint],
) -> Vec<(LintSeverity, String)> {
    constraints
        .iter()
        .filter(|c| !c.passed)
        .map(|c| {
            let severity = match c.strength {
                ConstraintStrength::Strong => LintSeverity::Error,
                ConstraintStrength::Medium => LintSeverity::Warning,
                ConstraintStrength::Weak => LintSeverity::Info,
            };
            (severity, c.message.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_text_protocol_is_optimal() {
        let features = ProtocolFeatures {
            has_text_contract: true,
            has_binary_store: false,
            is_self_contained: true,
            has_type_safety: true,
            has_multi_level: true,
            has_content_hash: false,
            has_entanglement: true,
        };
        let (optimal, _) = evaluate_protocol(&features);
        assert!(optimal, "pure text .tomllmd should be optimal");
    }

    #[test]
    fn binary_content_hash_is_optimal() {
        let features = ProtocolFeatures {
            has_text_contract: false,
            has_binary_store: true,
            is_self_contained: false,
            has_type_safety: true,
            has_multi_level: false,
            has_content_hash: true,
            has_entanglement: false,
        };
        let (optimal, _) = evaluate_protocol(&features);
        // P=0, B=1 → O = 1 (XOR is satisfied)
        assert!(optimal, "binary-only with content hash: P=0,B=1 → O=1");
    }

    #[test]
    fn violations_classify_correctly() {
        let features = ProtocolFeatures::default();
        let (_, constraints) = evaluate_protocol(&features);
        let violations = classify_violations(&constraints);
        let has_error = violations.iter().any(|(s, _)| *s == LintSeverity::Error);
        assert!(has_error, "no-protocol should produce Error-level violation");
    }

    #[test]
    fn multi_level_without_entanglement_generates_info() {
        let features = ProtocolFeatures {
            has_text_contract: true,
            has_binary_store: false,
            is_self_contained: true,
            has_type_safety: true,
            has_multi_level: true,
            has_content_hash: false,
            has_entanglement: false,
        };
        let (optimal, _) = evaluate_protocol(&features);
        assert!(optimal, "still optimal, just sub-optimal");
    }

    #[test]
    fn hybrid_text_and_binary_is_not_optimal() {
        let features = ProtocolFeatures {
            has_text_contract: true,
            has_binary_store: true,
            is_self_contained: false,
            has_type_safety: true,
            has_multi_level: true,
            has_content_hash: true,
            has_entanglement: true,
        };
        let (optimal, constraints) = evaluate_protocol(&features);
        assert!(!optimal, "hybrid text+binary should not be optimal");
        assert!(has_unrecoverable_violations(&constraints));
    }

    #[test]
    fn no_protocol_is_not_optimal() {
        let features = ProtocolFeatures::default();
        let (optimal, constraints) = evaluate_protocol(&features);
        assert!(!optimal);
        assert!(has_unrecoverable_violations(&constraints));
    }

    #[test]
    fn text_no_types_is_optimal_with_warnings() {
        let features = ProtocolFeatures {
            has_text_contract: true,
            has_binary_store: false,
            is_self_contained: true,
            has_type_safety: false,
            has_multi_level: false,
            has_content_hash: false,
            has_entanglement: false,
        };
        let (optimal, constraints) = evaluate_protocol(&features);
        assert!(optimal, "P=1,S=1,B=0 → O=1 per Z3");
        let violations = classify_violations(&constraints);
        assert!(violations.iter().any(|(s, _)| *s == LintSeverity::Warning));
    }

    #[test]
    fn binary_with_text_stub_not_optimal() {
        let features = ProtocolFeatures {
            has_text_contract: true,
            has_binary_store: true,
            is_self_contained: false,
            has_type_safety: true,
            has_multi_level: false,
            has_content_hash: true,
            has_entanglement: false,
        };
        let (optimal, _) = evaluate_protocol(&features);
        assert!(!optimal, "text+bin hybrid fails XOR constraint");
    }

    #[test]
    fn binary_store_requires_content_hash() {
        let features = ProtocolFeatures {
            has_text_contract: false,
            has_binary_store: true,
            is_self_contained: false,
            has_type_safety: true,
            has_multi_level: false,
            has_content_hash: false,
            has_entanglement: false,
        };
        let (optimal, constraints) = evaluate_protocol(&features);
        assert!(optimal, "binary-only satisfies XOR: P=0,B=1 → O=1");
        let warnings: Vec<_> = constraints.iter().filter(|c| !c.passed && c.strength == ConstraintStrength::Medium).collect();
        assert!(!warnings.is_empty(), "should warn about missing content hash");
    }
}
