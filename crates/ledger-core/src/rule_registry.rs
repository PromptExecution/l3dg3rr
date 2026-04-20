//! Rule Registry — stub module for multi-rule orchestration and semantic rule selection.
//!
//! ## Purpose
//! This module defines the interface for discovering, selecting, and applying Rhai
//! classification rules from a directory-based registry. It also defines the Rust-side
//! mirror types for the `reqif-opa-mcp` Python sidecar's JSON output.
//!
//! ## Status
//! - `RuleRegistry::load_from_dir` — STUB (unimplemented)
//! - `RuleRegistry::select_rules_deterministic` — STUB (unimplemented)
//! - `RuleRegistry::classify_waterfall` — STUB (unimplemented)
//! - `SemanticRuleSelector` — STUB (requires embedding infrastructure)
//!
//! ## External Dependency
//! The Python sidecar at <https://github.com/PromptExecution/reqif-opa-mcp> produces
//! `RequirementCandidate` JSON objects that are deserialized into `ReqIfCandidate` here.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::classify::{ClassificationEngine, ClassificationError, ClassificationOutcome, SampleTransaction};

// ============================================================================
// MIRROR TYPES: reqif-opa-mcp JSON output shapes
// ============================================================================

/// Mirrors `reqif-opa-mcp`'s `RequirementCandidate` JSON output.
///
/// Populated by calling the Python sidecar and deserializing its NDJSON output.
/// The Python pipeline produces these from a `DocumentGraph` after running through
/// the OPA gate. Each candidate represents a deterministically-derived requirement
/// from a source document.
///
/// # Sidecar pipeline
/// ```text
/// source PDF
///   → extract_docling_document
///   → DocumentGraph
///   → RequirementCandidate  ← serialized here
///   → OPA gate
///   → emit_reqif_xml
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReqIfCandidate {
    /// Stable key, e.g. `"REQ-001"` or a SHA-derived slug.
    pub key: String,
    /// Requirement text extracted from the source document.
    pub text: String,
    /// Section identifier within the source document (e.g., `"3.2.1"`).
    pub section: String,
    /// Human-readable rationale for why this was identified as a requirement.
    pub rationale: String,
    /// Source of the confidence score: `"rule"`, `"llm"`, `"heuristic"`, etc.
    pub confidence_source: String,
    /// Confidence in [0.0, 1.0].
    pub confidence: f64,
}

/// A document chunk with text and semantic anchoring.
///
/// Maps to `reqif-opa-mcp`'s `DocumentNode` — a canonical graph node that carries
/// extracted text, its parent in the document tree, a semantic identifier, and
/// positional anchors into the source PDF.
///
/// `DocumentChunk` objects are produced by the Python sidecar during the
/// `extract_docling_document` → `DocumentGraph` phase and streamed to Rust via
/// NDJSON over a subprocess pipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentChunk {
    /// Unique node identifier within the document graph.
    pub node_id: String,
    /// Extracted text content of this chunk.
    pub text: String,
    /// Parent node ID in the document tree (`None` for root chunks).
    pub parent_id: Option<String>,
    /// Semantic identifier: section number, heading slug, etc.
    pub semantic_id: String,
    /// Page anchors `[page_number, offset_chars]` into the source PDF.
    pub anchors: Vec<[u32; 2]>,
}

// ============================================================================
// ERRORS
// ============================================================================

/// Errors arising from rule registry operations.
#[derive(Debug, thiserror::Error)]
pub enum RuleRegistryError {
    #[error("failed to read rules directory: {0}")]
    Io(#[from] std::io::Error),

    #[error("no rules found in directory: {0}")]
    NoRules(PathBuf),

    #[error("classification error in waterfall: {0}")]
    Classification(#[from] ClassificationError),
}

// ============================================================================
// TRAIT: Semantic rule selection
// ============================================================================

/// Selects applicable Rhai rule files for a given transaction based on
/// vector similarity to `ReqIfCandidate` embeddings.
///
/// # Why this is `unimplemented!()` now
/// This trait requires an embedding model to encode both transaction descriptions
/// and `ReqIfCandidate` text into a shared vector space. The infrastructure for
/// local embedding inference (e.g., `candle`, `fastembed-rs`, or an ONNX sidecar)
/// is not yet wired. Until then, `RuleRegistry::select_rules_deterministic` provides
/// a keyword-match fallback that covers the common cases without embeddings.
pub trait SemanticRuleSelector {
    /// Select rules applicable to a transaction using vector similarity search.
    ///
    /// Returns rule file paths sorted by cosine similarity to the transaction's
    /// embedding, descending. At most `top_k` results are returned.
    ///
    /// # Prerequisites
    /// - Embedding index must be pre-built via `build_embedding_index`.
    /// - `ReqIfCandidate` objects must already be loaded into the registry.
    fn select_rules_semantic(
        &self,
        tx: &SampleTransaction,
        top_k: usize,
    ) -> Vec<PathBuf>;

    /// Build or rebuild the embedding index from loaded `ReqIfCandidate` texts.
    ///
    /// Must be called after `load_from_dir` and before `select_rules_semantic`.
    /// STUB: requires embedding infrastructure to implement.
    fn build_embedding_index(&mut self) -> Result<(), RuleRegistryError>;
}

// ============================================================================
// STRUCT: RuleRegistry
// ============================================================================

/// Registry of Rhai rule files with their associated `ReqIfCandidate` metadata.
///
/// Rules are loaded from a `rules/` directory at startup. Each `.rhai` file
/// represents one classification rule. Optionally, a paired `.reqif.json`
/// sidecar (produced by the Python `reqif-opa-mcp` pipeline) associates
/// `ReqIfCandidate` objects with each rule file.
///
/// # Production pipeline (waterfall model)
/// 1. `load_from_dir` — discover all `.rhai` files
/// 2. `select_rules_deterministic` — keyword-match to narrow the candidate set
/// 3. `classify_waterfall` — run rules in order; first non-`Unclassified` wins
///
/// # Planned: Semantic pipeline
/// Once embedding infrastructure is available, `SemanticRuleSelector` will replace
/// step 2 with vector similarity over `ReqIfCandidate` embeddings.
pub struct RuleRegistry {
    /// Paths to discovered `.rhai` rule files, sorted alphabetically.
    rule_paths: Vec<PathBuf>,
    /// Optional `ReqIfCandidate` objects associated with each rule, indexed
    /// parallel to `rule_paths`. `None` if no sidecar JSON was found.
    candidates: Vec<Option<ReqIfCandidate>>,
}

impl RuleRegistry {
    /// Load all `.rhai` files from a rules directory.
    ///
    /// Scans `rules_dir` for files ending in `.rhai` and sorts them alphabetically.
    /// Optionally loads a paired `<rule_name>.reqif.json` sidecar for each rule.
    ///
    /// Returns `RuleRegistryError::NoRules` if the directory contains no `.rhai` files.
    ///
    /// # Stub
    /// This method is not yet implemented.
    pub fn load_from_dir(_rules_dir: &Path) -> Result<Self, RuleRegistryError> {
        unimplemented!("RuleRegistry::load_from_dir — scan directory for .rhai files and optional .reqif.json sidecars")
    }

    /// Select rules applicable to a transaction by keyword match (deterministic fallback).
    ///
    /// Filters `rule_paths` by checking whether the rule file name or its associated
    /// `ReqIfCandidate` text contains any keyword from the transaction's description.
    /// Returns rule file paths sorted by expected relevance (match count descending).
    ///
    /// This is the deterministic fallback used when embedding infrastructure is absent.
    /// All rules are returned (in alphabetical order) when no keyword match is found,
    /// ensuring the waterfall always has candidates.
    ///
    /// # Stub
    /// This method is not yet implemented.
    pub fn select_rules_deterministic(&self, _tx: &SampleTransaction) -> Vec<PathBuf> {
        unimplemented!("RuleRegistry::select_rules_deterministic — keyword-match rule selection from registry")
    }

    /// Apply all rules in order, returning the first non-`Unclassified` result.
    ///
    /// This is the production multi-rule pipeline (waterfall model). Rules are
    /// executed in the order returned by `select_rules_deterministic`. Execution
    /// stops as soon as one rule returns a `category` other than `"Unclassified"`.
    ///
    /// If all rules return `"Unclassified"`, the last result is returned so callers
    /// always receive a `ClassificationOutcome`.
    ///
    /// # Stub
    /// This method is not yet implemented.
    pub fn classify_waterfall(
        &self,
        _engine: &mut ClassificationEngine,
        _tx: &SampleTransaction,
    ) -> Result<ClassificationOutcome, ClassificationError> {
        unimplemented!("RuleRegistry::classify_waterfall — multi-rule waterfall: first non-Unclassified result wins")
    }

    /// Return the number of rules loaded in this registry.
    pub fn rule_count(&self) -> usize {
        self.rule_paths.len()
    }

    /// Return the rule paths in registry order.
    pub fn rule_paths(&self) -> &[PathBuf] {
        &self.rule_paths
    }
}

impl SemanticRuleSelector for RuleRegistry {
    fn select_rules_semantic(&self, _tx: &SampleTransaction, _top_k: usize) -> Vec<PathBuf> {
        unimplemented!(
            "SemanticRuleSelector::select_rules_semantic — requires embedding infrastructure \
             (local ONNX / fastembed-rs / candle model); use select_rules_deterministic until available"
        )
    }

    fn build_embedding_index(&mut self) -> Result<(), RuleRegistryError> {
        unimplemented!(
            "SemanticRuleSelector::build_embedding_index — requires embedding model; \
             blocked until embedding infrastructure is wired"
        )
    }
}
