//! Integration tests for the l3dg3rr tax-ledger pipeline.
//!
//! All tests in this module are marked `#[ignore]` because they depend on
//! infrastructure or APIs not yet implemented. They are designed to:
//!   1. Compile without error against current types
//!   2. Fail at runtime (either via `unimplemented!()` panic or an assertion on
//!      `LedgerOpError::NotImplemented`)
//!   3. Document the desired behavior in detail so future implementors have
//!      unambiguous acceptance criteria
//!
//! Run all ignored tests (expect failures) with:
//!   cargo test -p ledger-core --test integration_tests -- --ignored

#[cfg(test)]
mod integration {
    use std::path::PathBuf;
    use std::sync::Arc;

    // -------------------------------------------------------------------------
    // Test #4 — Calendar drives OperationDispatcher
    // -------------------------------------------------------------------------

    /// Verify that a `BusinessCalendar` can be the sole source of truth for
    /// constructing an `OperationDispatcher`.
    ///
    /// # What needs to be built first
    /// `OperationDispatcher::from_scheduled_events(&[ScheduledEvent])` — a
    /// constructor that iterates the event list, maps each event's `operation`
    /// field to a concrete `Box<dyn LedgerOperation>`, and registers them so
    /// `run_by_id(event.id)` dispatches the correct op.
    #[test]
    fn test_calendar_drives_operation_dispatcher() {
        use crate::calendar::BusinessCalendar;
        use crate::ledger_ops::{OperationContext, OperationDispatcher};

        let cal = BusinessCalendar::us_tax_defaults();
        let dispatcher = OperationDispatcher::from_scheduled_events(&cal.events);

        let ctx = OperationContext::new(
            PathBuf::from("/tmp/working"),
            PathBuf::from("/tmp/rules"),
        );

        let result = dispatcher.run_by_id("us-quarterly-estimated", &ctx);
        assert!(
            result.is_some(),
            "dispatcher should have an op keyed by event id 'us-quarterly-estimated'"
        );
        assert!(
            result.unwrap().is_ok(),
            "CheckTaxDeadlineOp should return Ok"
        );
    }

    // Tests #5a and #5b moved to crates/ledgerr-mcp/tests/tools.rs where they
    // can import TOOL_REGISTRY directly from the ledgerr-mcp crate.

    // -------------------------------------------------------------------------
    // Test #6 — PDF ingest via subprocess sidecar
    // -------------------------------------------------------------------------

    /// Verify that `IngestStatementOp::execute()` can process a fixture PDF via
    /// the Docling sidecar and produce at least one ingested transaction row.
    ///
    /// # What needs to be built first
    /// Phase-2 work: `IngestStatementOp::execute()` must:
    ///   - Spawn `docling --pdf <path> --output ndjson` (or equivalent)
    ///   - Parse NDJSON stdout into transaction rows
    ///   - Compute Blake3 content-hash IDs
    ///   - Return `OperationResult { success: true, items_processed: N }`
    ///
    /// Also requires: `tests/fixtures/sample_hsbc_statement.pdf`
    #[test]
    #[ignore = "requires IngestStatementOp::execute() subprocess wiring — phase-2 work; also needs fixture PDF"]
    fn test_ingest_statement_via_pdf_sidecar() {
        // DESIRED BEHAVIOR:
        // IngestStatementOp::execute() should:
        //   1. Glob ctx.working_dir / self.source_glob for PDF files
        //   2. For each file, spawn the Docling sidecar CLI:
        //        docling --pdf <path> --output ndjson
        //   3. Read NDJSON lines from stdout; deserialize each as a transaction row
        //   4. Compute Blake3 ID: blake3(account_id + date + amount + description)
        //   5. Upsert rows (skip duplicates by hash)
        //   6. Return OperationResult { success: true, items_processed: rows_seen,
        //                               items_flagged: rows_needing_review }
        //
        // The fixture at tests/fixtures/sample_hsbc_statement.pdf should contain
        // exactly one transaction line for deterministic test assertions.
        use crate::ledger_ops::{IngestStatementOp, LedgerOperation, LedgerOpError, OperationContext};

        let op = IngestStatementOp {
            source_glob: "tests/fixtures/*.pdf".to_string(),
            vendor_hint: Some("HSBC".to_string()),
        };

        // Point working_dir at the repo root so the glob resolves correctly.
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap() // crates/ledger-core → crates
            .parent().unwrap() // crates → repo root
            .to_path_buf();

        let ctx = OperationContext::new(repo_root, PathBuf::from("/tmp/rules"));

        let result = op.execute(&ctx);

        // Current expectation: returns NotImplemented (phase-1 stub)
        // Future expectation after phase-2: returns Ok with items_processed > 0
        match &result {
            Err(LedgerOpError::NotImplemented(_)) => {
                panic!(
                    "IngestStatementOp still returns NotImplemented — implement PDF sidecar \
                     subprocess call in phase-2 to make this test pass"
                );
            }
            Ok(op_result) if !op_result.success => {
                panic!("PDF ingest returned success=false: {:?}", op_result.issues);
            }
            Ok(op_result) => {
                assert!(
                    op_result.items_processed > 0,
                    "should have ingested at least one row from fixture PDF; got 0"
                );
            }
            Err(e) => panic!("unexpected error during PDF ingest: {e:?}"),
        }
    }

    // -------------------------------------------------------------------------
    // Test #7 — OPA gate filters requirement candidates
    // -------------------------------------------------------------------------

    /// Verify that an OPA gate operation can route low-confidence classifications
    /// to FLAGS.open instead of their target schedule sheet.
    ///
    /// # What needs to be built first
    /// Phase-3 work: A new `OpaGateOp` struct implementing `LedgerOperation` that:
    ///   - POSTs each `ClassificationOutcome` to OPA at `localhost:8181`
    ///   - Sends body: `{ "input": { "category": "...", "confidence": 0.9, "review": true } }`
    ///   - If OPA returns `{ "result": false }` → move tx to FLAGS.open
    ///   - Requires OPA server: `opa run --server opa/policies/ledger_classify.rego`
    #[test]
    #[ignore = "requires OpaGateOp (not yet implemented) and a running OPA sidecar at localhost:8181 — phase-3 work"]
    fn test_opa_gate_filters_requirement_candidates() {
        // DESIRED BEHAVIOR:
        // OpaGateOp::execute(&ctx) should:
        //   1. Load pending ClassificationOutcome objects from ctx.working_dir
        //   2. For each outcome, POST to OPA:
        //        POST http://localhost:8181/v1/data/ledger/allow
        //        { "input": { "category": "ForeignIncome", "confidence": 0.9, "review": true } }
        //   3. If response.result == false → flag the transaction (move to FLAGS.open)
        //   4. If OPA is unreachable → log warning, do NOT hard-fail the pipeline
        //      (return OperationResult { success: true, items_flagged: 0, issues: ["OPA unreachable"] })
        //
        // Test setup: the OPA policy at opa/policies/ledger_classify.rego must
        // deny any transaction with confidence < 0.75.
        // After the op runs, transactions with confidence < 0.75 should appear
        // in FLAGS.open and not in their schedule sheet.
        //
        // This test uses ClassifyTransactionsOp output as input to OpaGateOp,
        // demonstrating the pipeline composition:
        //   IngestStatementOp → ClassifyTransactionsOp → OpaGateOp → ExportWorkbookOp
        use crate::ledger_ops::{ClassifyTransactionsOp, LedgerOperation, LedgerOpError, OperationContext};

        // ClassifyTransactionsOp is the nearest existing op; OpaGateOp doesn't exist yet.
        // This stub exercises the existing op to prove pipeline composition compiles.
        let classify_op = ClassifyTransactionsOp {
            rule_dir: PathBuf::from("/tmp/rules"),
            review_threshold: 0.75,
            account_filter: None,
        };

        let ctx = OperationContext::new(
            PathBuf::from("/tmp/working"),
            PathBuf::from("/tmp/rules"),
        );

        let classify_result = classify_op.execute(&ctx);

        // Current: NotImplemented. Future: Ok after phase-2+3 wiring.
        match classify_result {
            Err(LedgerOpError::NotImplemented(_)) => {
                panic!(
                    "ClassifyTransactionsOp still returns NotImplemented — implement Rhai \
                     engine integration (phase-2), then add OpaGateOp (phase-3)"
                );
            }
            Ok(result) => {
                // Once OpaGateOp exists, chain it here:
                //   let opa_op = OpaGateOp::new();
                //   let opa_result = opa_op.execute(&ctx).unwrap();
                //   assert!(opa_result.success, "OPA gate should not hard-fail");
                assert!(
                    result.success,
                    "classify op should succeed before OPA gate can run"
                );
            }
            Err(e) => panic!("unexpected classify error: {e:?}"),
        }
    }

    // -------------------------------------------------------------------------
    // Test #8 — LLM verification proposes a repair for a classification outcome
    // -------------------------------------------------------------------------

    // Uses MockModelClient for deterministic coverage.
    // Replace proposer/reviewer with AnthropicModelClient for live LLM coverage.
    #[test]
    fn test_llm_verification_proposes_category() {
        use crate::verify::{MockModelClient, MultiModelConfig, MultiModelVerifier, VerificationOutcome};
        let proposer_json = r#"{
            "rule_id": "ForeignIncome",
            "proposed_fix": "ForeignIncome",
            "reasoning": "Wire transfer from foreign employer matches ForeignIncome pattern",
            "confidence": 0.92
        }"#;
        let reviewer_json =
            r#"{"approved":true,"concerns":[],"suggestions":[],"confidence":0.90}"#;

        let proposer = MockModelClient::default().with_response(proposer_json);
        let reviewer = MockModelClient::default().with_response(reviewer_json);

        let config = MultiModelConfig::new(
            "claude-haiku-4-5-20251001",
            "claude-haiku-4-5-20251001",
        )
        .with_threshold(0.80);

        let verifier = MultiModelVerifier::new(proposer, reviewer, config);

        // issues_json represents a classification outcome that needs repair
        let issues_json = r#"[{"field":"category","value":"Unclassified","confidence":0.3}]"#;
        let context = "transaction: {account_id: HSBC-INTL-001, description: Wire transfer from DE employer, amount: 5000.00}";

        let outcome = verifier
            .verify("ForeignIncome", issues_json, context)
            .expect("verifier should not error with mock clients");

        assert!(
            outcome.is_approved(),
            "mock models should agree and approve; if using real models they may disagree — \
             that is expected behavior, not a bug"
        );

        match outcome {
            VerificationOutcome::Approved { proposal, review } => {
                assert!(
                    !proposal.rule_id.is_empty(),
                    "proposal.rule_id (category) must not be empty"
                );
                assert!(
                    proposal.confidence > 0.0 && proposal.confidence <= 1.0,
                    "proposal.confidence must be in (0, 1]"
                );
                assert!(
                    review.confidence > 0.0 && review.confidence <= 1.0,
                    "review.confidence must be in (0, 1]"
                );
            }
            VerificationOutcome::Rejected { proposal, review } => {
                panic!(
                    "verifier rejected proposal — proposer said {:?}, reviewer said {:?}",
                    proposal.rule_id, review.concerns
                );
            }
        }
    }

    // -------------------------------------------------------------------------
    // Test #9 — Semantic rule selector selects by embedding
    // -------------------------------------------------------------------------

    /// Verify that `SemanticRuleSelector::select_rules_semantic()` can match a
    /// German-language transaction description to the correct Rhai rule file
    /// without any keyword overlap.
    ///
    /// # What needs to be built first
    /// `RuleRegistry::load_from_dir()` (currently `unimplemented!()`) and
    /// `SemanticRuleSelector::build_embedding_index()` (also `unimplemented!()`).
    /// Both require embedding infrastructure (fastembed-rs, candle, or ONNX sidecar).
    #[test]
    #[ignore = "requires RuleRegistry::load_from_dir() and SemanticRuleSelector::build_embedding_index() — both unimplemented!() panic; blocked on embedding infrastructure"]
    fn test_semantic_rule_selector_selects_by_embedding() {
        // DESIRED BEHAVIOR:
        // 1. RuleRegistry::load_from_dir(&rules_dir) must:
        //    - Scan rules/ for *.rhai files
        //    - Optionally load *.reqif.json sidecars
        //    - Return a populated RuleRegistry (no unimplemented!() panic)
        //
        // 2. registry.build_embedding_index() must:
        //    - Encode each rule file's content (or its ReqIfCandidate.text) via
        //      a local embedding model into a shared vector space
        //    - Build a k-d tree or flat cosine-similarity index over the vectors
        //
        // 3. registry.select_rules_semantic(&tx, 3) must:
        //    - Encode tx.description ("Auslandüberweisung von DE Arbeitgeber")
        //    - Return the top-3 rule paths by cosine similarity
        //    - "Auslandüberweisung" (German: "foreign transfer") should match
        //      classify_foreign_income.rhai even though the German word is not a
        //      keyword in the rule file — this validates semantic (not lexical) matching
        //
        // The test asserts that at least one returned path contains "foreign_income"
        // in its filename, proving the semantic index correctly bridges languages.
        use crate::classify::SampleTransaction;
        use crate::rule_registry::{RuleRegistry, SemanticRuleSelector};

        let rule_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap() // crates/ledger-core → crates
            .parent()
            .unwrap() // crates → repo root
            .join("rules");

        // This will panic with unimplemented!() until load_from_dir is implemented:
        let mut registry =
            RuleRegistry::load_from_dir(&rule_dir).expect("should load rules from rules/ dir");

        // This will panic with unimplemented!() until build_embedding_index is implemented:
        registry
            .build_embedding_index()
            .expect("should build embedding index over rule files");

        let tx = SampleTransaction {
            tx_id: "test-semantic-001".to_string(),
            account_id: "HSBC-DE-001".to_string(),
            date: "2024-03-15".to_string(),
            amount: "3200.00".to_string(),
            description: "Auslandüberweisung von DE Arbeitgeber".to_string(),
        };

        // top_k = 5: return up to 5 most semantically similar rules
        let selected = registry.select_rules_semantic(&tx, 5);

        assert!(
            !selected.is_empty(),
            "semantic selector must return at least one rule for a foreign transfer description"
        );

        let names: Vec<&str> = selected
            .iter()
            .filter_map(|p| p.file_name()?.to_str())
            .collect();

        assert!(
            names.iter().any(|n| n.contains("foreign_income")),
            "expected classify_foreign_income.rhai in top-5 semantic matches for \
             'Auslandüberweisung von DE Arbeitgeber'; got: {names:?}\n\
             This means the embedding model did NOT map the German 'Auslandüberweisung' \
             close enough to the English 'foreign income' vector space."
        );
    }
}
