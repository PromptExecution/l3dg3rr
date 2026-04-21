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
    #[ignore = "requires OperationDispatcher::from_scheduled_events() — not yet implemented"]
    fn test_calendar_drives_operation_dispatcher() {
        // DESIRED BEHAVIOR:
        // BusinessCalendar::us_tax_defaults() returns a populated calendar whose
        // events Vec contains at least "us-quarterly-estimated".
        //
        // OperationDispatcher::from_scheduled_events(&cal.events) should:
        //   - Iterate events
        //   - For OperationKind::CheckTaxDeadline { deadline_id } → register a
        //     CheckTaxDeadlineOp { deadline_id, warn_days_before: 30 }
        //   - For OperationKind::IngestStatement { source_glob } → register an
        //     IngestStatementOp { source_glob, vendor_hint: None }
        //   - Key each operation by the ScheduledEvent::id (not op.id())
        //
        // Running dispatcher.run_by_id("us-quarterly-estimated", &ctx) should
        // return Some(Err(LedgerOpError::NotImplemented(...))) today, and
        // Some(Ok(OperationResult { success: true, ... })) once the op is wired.
        //
        // This test asserts the FUTURE state: the result should be Some and Ok.
        use crate::calendar::BusinessCalendar;
        use crate::ledger_ops::{LedgerOpError, OperationContext, OperationDispatcher};

        let cal = BusinessCalendar::us_tax_defaults();

        // OperationDispatcher::from_scheduled_events does not exist yet —
        // calling it will produce a compile error until implemented.
        // For now we simulate the desired shape with a placeholder that panics:
        let _dispatcher: OperationDispatcher = {
            // Placeholder: in real implementation this would be:
            //   OperationDispatcher::from_scheduled_events(&cal.events)
            // For compile-time correctness we build a stub that will fail the
            // assertion below.
            let mut d = OperationDispatcher::new();
            // Register nothing — run_by_id will return None, failing the assert.
            // Once from_scheduled_events is implemented, replace this block.
            let _ = &cal.events; // use cal to avoid unused warning
            d
        };

        let ctx = OperationContext::new(
            PathBuf::from("/tmp/working"),
            PathBuf::from("/tmp/rules"),
        );

        // When from_scheduled_events is implemented this should find the op:
        let result = _dispatcher.run_by_id("us-quarterly-estimated", &ctx);
        assert!(
            result.is_some(),
            "calendar-driven dispatcher should have an op for 'us-quarterly-estimated'; \
             got None — OperationDispatcher::from_scheduled_events not yet implemented"
        );

        // Once the full op is wired, this should succeed:
        match result.unwrap() {
            Ok(op_result) => {
                assert!(
                    op_result.success,
                    "CheckTaxDeadlineOp should report success when calendar is attached"
                );
            }
            Err(LedgerOpError::NotImplemented(msg)) => {
                panic!(
                    "CheckTaxDeadlineOp still returns NotImplemented: {msg}\n\
                     Implement calendar lookup in CheckTaxDeadlineOp::execute()"
                );
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }

    // -------------------------------------------------------------------------
    // Test #5a — MCP calendar events tool exists
    // -------------------------------------------------------------------------

    /// Verify that the `ledgerr-mcp` crate exposes a `list_calendar_events` tool.
    ///
    /// # What needs to be built first
    /// The `ledgerr-mcp` crate (does not yet exist as a separate crate) must:
    ///   - Define a `TOOL_REGISTRY: &[&str]` constant (or equivalent)
    ///   - Register a tool named `"list_calendar_events"` that calls
    ///     `BusinessCalendar::us_tax_defaults()` and returns a JSON array of
    ///     `ScheduledEvent` objects with fields:
    ///       id, description, next_due_date, jurisdiction
    #[test]
    #[ignore = "requires MCP tool 'list_calendar_events' to be registered in ledgerr-mcp crate"]
    fn test_mcp_list_calendar_events_tool_exists() {
        // DESIRED BEHAVIOR:
        // use ledgerr_mcp::tools::TOOL_REGISTRY;
        // assert!(TOOL_REGISTRY.contains(&"list_calendar_events"), ...);
        //
        // The tool implementation should:
        //   1. Instantiate BusinessCalendar::us_tax_defaults()
        //   2. For each event, compute next_due_date using BusinessCalendar::next_due()
        //      with today's date as the `after` anchor
        //   3. Serialize to JSON array: [{id, description, next_due_date, jurisdiction}]
        //
        // This test will fail until `ledgerr-mcp` is created and the tool is registered.
        panic!(
            "ledgerr-mcp crate does not exist yet — create the crate, \
             implement TOOL_REGISTRY, and register 'list_calendar_events'"
        );
    }

    // -------------------------------------------------------------------------
    // Test #5b — MCP document shape tool exists
    // -------------------------------------------------------------------------

    /// Verify that the `ledgerr-mcp` crate exposes a `get_document_shape` tool.
    ///
    /// # What needs to be built first
    /// Same `ledgerr-mcp` crate requirement as test #5a. The tool must:
    ///   - Accept `{ filename: String, sample_content: String }`
    ///   - Return a `DocumentShape` with fields:
    ///       vendor, column_map, date_format, jurisdiction
    #[test]
    #[ignore = "requires MCP tool 'get_document_shape' to be registered in ledgerr-mcp crate"]
    fn test_mcp_get_document_shape_tool_exists() {
        // DESIRED BEHAVIOR:
        // use ledgerr_mcp::tools::TOOL_REGISTRY;
        // assert!(TOOL_REGISTRY.contains(&"get_document_shape"), ...);
        //
        // The tool implementation should:
        //   1. Parse the filename using FilenameParser (from crate::filename)
        //      to extract vendor hint
        //   2. Run document shape detection via ClassificationEngine or
        //      dedicated shape detection logic
        //   3. Return { vendor, column_map: { date: usize, amount: usize, desc: usize },
        //               date_format: "%Y-%m-%d", jurisdiction: "US" | "AU" | ... }
        //
        // This test will fail until `ledgerr-mcp` is created and the tool is registered.
        panic!(
            "ledgerr-mcp crate does not exist yet — create the crate, \
             implement TOOL_REGISTRY, and register 'get_document_shape'"
        );
    }

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

    /// Verify that `MultiModelVerifier` can propose and review a classification
    /// repair using real LLM models when `ANTHROPIC_API_KEY` is set.
    ///
    /// # What needs to be built first
    /// A real `AnthropicModelClient` implementing `ModelClient` that calls the
    /// Anthropic messages API. The existing `MockModelClient` is synchronous and
    /// test-only; a production client needs async-over-sync bridging or a tokio
    /// runtime handle.
    #[test]
    #[ignore = "requires a real AnthropicModelClient implementing ModelClient — not yet implemented; also needs ANTHROPIC_API_KEY"]
    fn test_llm_verification_proposes_category() {
        // DESIRED BEHAVIOR:
        // When ANTHROPIC_API_KEY is set, MultiModelVerifier::new(proposer, reviewer, config)
        // where proposer and reviewer are AnthropicModelClient instances should:
        //   1. proposer.complete(prompt) → sends a Messages API request to Claude
        //      with the transaction fields + preliminary category as context
        //   2. reviewer.complete(proposal_json) → evaluates the proposal
        //   3. verify() returns VerificationOutcome::Approved when both models agree
        //      on a category and reviewer confidence >= min_reviewer_confidence
        //
        // The VerificationOutcome::Approved variant carries:
        //   - proposal.rule_id (maps to transaction category)
        //   - proposal.proposed_fix (maps to corrected category label)
        //   - review.confidence (in [0.0, 1.0])
        //
        // For transactions like "Wire transfer from DE employer", the proposer
        // should propose "ForeignIncome" and the reviewer should approve it.
        //
        // This test uses ANTHROPIC_API_KEY env var and skips if absent.
        use crate::verify::{MockModelClient, MultiModelConfig, MultiModelVerifier, VerificationOutcome};

        if std::env::var("ANTHROPIC_API_KEY").is_err() {
            // Skip gracefully — this test requires real API credentials
            return;
        }

        // TODO: Replace MockModelClient with AnthropicModelClient once implemented.
        // AnthropicModelClient::new(api_key, model_id) is the desired constructor.
        // Using mock here so the test compiles; it will NOT exercise real LLM behavior.
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
