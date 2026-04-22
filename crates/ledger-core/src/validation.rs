//! Validation types for the verb pipeline.
//! These types provide a carry-forward validation context that accumulates
//! confidence and issues through each pipeline stage.

use serde::{Deserialize, Serialize};

/// Disposition classifies how an issue should be handled by the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disposition {
    /// Pipeline must stop. No recovery possible. Examples: zero-amount, corrupt source.
    Unrecoverable,
    /// Pipeline may continue with degraded confidence. Future rules may fix this.
    /// Examples: wrong tax code, unusual amount, ambiguous vendor.
    Recoverable,
    /// Not an error. Informational context only. Examples: new vendor, first of month.
    Advisory,
}

/// Source identifies which validation layer produced an issue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueSource {
    /// T1 — Rust type boundary check. Always unrecoverable.
    TypeCheck,
    /// T2 — Kasuari constraint solver. Recoverable based on strength.
    Constraint {
        /// Strength of constraint that failed (0.0-1.0).
        strength: f32,
    },
    /// T3 — Rhai rule evaluation.
    RhaiRule {
        /// Rule file or identifier that generated this issue.
        rule_id: String,
    },
}

/// A single validation issue produced by a pipeline stage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Field that caused the issue, if applicable.
    pub field: Option<String>,
    /// How this issue should be handled.
    pub disposition: Disposition,
    /// Which validation layer produced this.
    pub source: IssueSource,
}

impl Issue {
    pub fn unrecoverable(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: msg.into(),
            field: None,
            disposition: Disposition::Unrecoverable,
            source: IssueSource::TypeCheck,
        }
    }

    pub fn recoverable(
        code: impl Into<String>,
        msg: impl Into<String>,
        source: IssueSource,
    ) -> Self {
        Self {
            code: code.into(),
            message: msg.into(),
            field: None,
            disposition: Disposition::Recoverable,
            source,
        }
    }

    pub fn advisory(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: msg.into(),
            field: None,
            disposition: Disposition::Advisory,
            source: IssueSource::RhaiRule {
                rule_id: "advisory".to_string(),
            },
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }
}

/// Accumulated state flowing forward through the pipeline.
/// Each stage reads and extends this context.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MetaCtx {
    /// Compound confidence across all previous stages (0.0-1.0).
    /// Degrades multiplicatively as issues accumulate.
    pub accumulated_confidence: f32,
    /// Flags set by any upstream stage.
    pub flags: Vec<MetaFlag>,
    /// Trace of each stage's confidence score for audit.
    pub stage_trace: Vec<StageScore>,
}

/// Flags set by stages, readable by downstream stages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetaFlag {
    /// New vendor never seen before.
    NewVendor { vendor: String },
    /// Data point is anomalous based on historical constraints.
    AnomalyDetected { code: String, impact: f32 },
    /// A Rhai rule was automatically repaired.
    RepairApplied { rule_id: String },
    /// Upstream stage produced low confidence.
    LowUpstreamConf { score: f32, stage: String },
    /// Constraint solver found weak satisfaction.
    ConstraintWeak { constraint: String },
}

/// Score from a single pipeline stage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StageScore {
    pub stage: String,
    pub confidence: f32,
    pub issue_count: usize,
}

impl MetaCtx {
    /// Advance context with a new stage's results.
    pub fn advance(
        &self,
        stage: &str,
        stage_confidence: f32,
        issues: &[Issue],
    ) -> Self {
        let mut next = self.clone();

        // Compound probability — accumulated_confidence degrades multiplicatively
        next.accumulated_confidence = if self.accumulated_confidence == 0.0 {
            stage_confidence
        } else {
            self.accumulated_confidence * stage_confidence
        };

        next.stage_trace.push(StageScore {
            stage: stage.to_string(),
            confidence: stage_confidence,
            issue_count: issues.len(),
        });

        // Promote recoverable issues into MetaFlags
        for _issue in issues.iter().filter(|i| {
            matches!(i.disposition, Disposition::Recoverable)
        }) {
            next.flags.push(MetaFlag::LowUpstreamConf {
                score: stage_confidence,
                stage: stage.to_string(),
            });
        }

        next
    }

    /// Create initial context for a pipeline run.
    pub fn initial() -> Self {
        Self::default()
    }
}

/// Result of a pipeline stage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StageResult<T> {
    /// The stage's output data.
    pub data: T,
    /// This stage's confidence score (0.0-1.0).
    pub confidence: f32,
    /// Issues produced by this stage.
    pub issues: Vec<Issue>,
    /// Updated context for next stage.
    pub meta: MetaCtx,
}

impl<T> StageResult<T> {
    /// Create a successful stage result.
    pub fn ok(data: T, confidence: f32) -> Self {
        Self {
            data,
            confidence,
            issues: Vec::new(),
            meta: MetaCtx::default(),
        }
    }

    /// Create a stage result with issues.
    pub fn with_issues(
        data: T,
        confidence: f32,
        issues: impl Into<Vec<Issue>>,
    ) -> Self {
        Self {
            data,
            confidence,
            issues: issues.into(),
            meta: MetaCtx::default(),
        }
    }
}

/// Pipe a stage result into the next stage, compounding confidence.
/// The closure receives the current stage's MetaCtx and returns the next stage result.
pub fn and_then<T, U, F>(current: StageResult<T>, stage: &str, f: F) -> StageResult<U>
where
    F: FnOnce(MetaCtx) -> StageResult<U>,
{
    let next = f(current.meta.clone());
    let issues = next.issues.clone();
    StageResult {
        data: next.data,
        confidence: next.confidence,
        issues: next.issues,
        meta: next.meta.advance(stage, next.confidence, &issues),
    }
}

/// Reversibility defines whether a verb can be undone and under what conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reversibility {
    /// Can be retried at will with no side effects.
    Free,
    /// Can be reversed but requires approval to do so.
    ReversibleWithAuth,
    /// Permanent — requires approval to execute.
    Irreversible,
}

/// Access criteria for verb execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccessCriteria {
    /// Any caller can execute.
    Open,
    /// Upstream confidence must meet minimum threshold.
    MinConfidence(f32),
    /// Requires approval before execution.
    RequiresApproval(ApprovalGate),
    /// Caller must hold this role.
    RequiresRole(String),
}

/// Approval gate types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalGate {
    /// Desktop notification, operator clicks approve.
    Tray,
    /// Second rig agent reviews before human sees it.
    DualModel,
    /// Human only, no model pre-review.
    Human,
}

/// Verb is the primary abstraction for pipeline actions.
/// Each verb performs one action with a defined input, output, reversibility, and access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbDef {
    pub name: String,
    pub input_schema: String,
    pub output_schema: String,
    pub reversible: Reversibility,
    pub access: AccessCriteria,
    /// Path to Rhai handler file.
    pub rhai_handler: Option<String>,
}

impl VerbDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            input_schema: String::new(),
            output_schema: String::new(),
            reversible: Reversibility::Free,
            access: AccessCriteria::Open,
            rhai_handler: None,
        }
    }

    pub fn with_input(mut self, schema: impl Into<String>) -> Self {
        self.input_schema = schema.into();
        self
    }

    pub fn with_output(mut self, schema: impl Into<String>) -> Self {
        self.output_schema = schema.into();
        self
    }

    pub fn with_access(mut self, access: AccessCriteria) -> Self {
        self.access = access;
        self
    }

    pub fn with_handler(mut self, path: impl Into<String>) -> Self {
        self.rhai_handler = Some(path.into());
        self
    }
}

/// Well-known verbs for the ledger pipeline.
pub mod verbs {
    use super::*;

    /// Detect: identify document shape from raw input.
    pub fn detect() -> VerbDef {
        VerbDef::new("detect").with_input("bytes").with_output("ShapeResult")
    }

    /// Validate: check data plausibility and constraints.
    pub fn validate() -> VerbDef {
        VerbDef::new("validate")
            .with_input("ShapeResult")
            .with_output("Validated")
    }

    /// Classify: assign account code and category.
    pub fn classify() -> VerbDef {
        VerbDef::new("classify")
            .with_input("Validated")
            .with_output("Classified")
    }

    /// Reconcile: prepare for backend (Xero, Excel).
    pub fn reconcile() -> VerbDef {
        VerbDef::new("reconcile")
            .with_input("Classified")
            .with_output("Posting")
    }

    /// Commit: permanently record (irreversible).
    pub fn commit() -> VerbDef {
        VerbDef::new("commit")
            .with_input("Posting")
            .with_output("LedgerEntry")
            .with_access(AccessCriteria::RequiresApproval(ApprovalGate::Tray))
    }

    /// Reverse: undo a previous commit.
    pub fn reverse() -> VerbDef {
        VerbDef::new("reverse")
            .with_input("LedgerEntry")
            .with_output("Reversal")
            .with_access(AccessCriteria::RequiresApproval(ApprovalGate::Tray))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_unrecoverable() {
        let issue = Issue::unrecoverable("zero_amount", "amount cannot be zero");
        assert_eq!(issue.disposition, Disposition::Unrecoverable);
        assert_eq!(issue.code, "zero_amount");
    }

    #[test]
    fn test_issue_recoverable() {
        let issue = Issue::recoverable(
            "unusual_amount",
            "amount outside historical range",
            IssueSource::Constraint { strength: 0.7 },
        );
        assert_eq!(issue.disposition, Disposition::Recoverable);
    }

    #[test]
    fn test_issue_advisory() {
        let issue = Issue::advisory("new_vendor", "vendor not seen before");
        assert_eq!(issue.disposition, Disposition::Advisory);
    }

    #[test]
    fn test_meta_ctx_compound_confidence() {
        let ctx = MetaCtx::initial();
        let ctx1 = ctx.advance("stage1", 0.9, &[]);
        assert_eq!(ctx1.accumulated_confidence, 0.9);

        let ctx2 = ctx1.advance("stage2", 0.8, &[]);
        assert!((ctx2.accumulated_confidence - 0.72).abs() < 0.001);
    }

    #[test]
    fn test_stage_result_progression() {
        let stage1 = StageResult::ok("input".to_string(), 0.95);
        assert_eq!(stage1.meta.accumulated_confidence, 0.0); // not yet advanced

        let stage2 = and_then(stage1, "validate", |ctx| {
            StageResult::ok("validated".to_string(), 0.9)
        });

        assert_eq!(stage2.data, "validated");
        assert!((stage2.meta.accumulated_confidence - 0.9).abs() < 0.001);
    }
}