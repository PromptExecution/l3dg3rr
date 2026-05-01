# PRD-6: Rust Edition 2024, Crate Modernization & Type-Level Abstractions

**Status:** Draft | **Priority:** P2 (Quality) | **Target:** Post-PRD-5 stabilization

---

## 1. Problem Statement

The codebase pins `edition = "2021"` with `edition = "2021"` explicitly on every crate, and uses Rust 2021 idioms throughout, despite `rustc 1.95.0` being the installed toolchain. This leaves ~60 stabilized language features, 150+ new std APIs, and two edition migrations (2024 is current) on the table.

Beyond the edition gap, the codebase contains distinct *architectural surfaces* that would each benefit from targeted crate adoption to make the application's unique value — Z3-audited financial logic, Rhai-scripted workflow self-visualization, type-state pipeline enforcement, content-hashed evidence graphs — *more visible, testable, and composable* through Rust's type system.

### Current friction points identified by deep-dive analysis

| Friction | Location | Lines | Cost |
|---|---|---|---|
| 3-deep `if let Some` chains in parser, tracer, adapter | `parser.rs`, `trace.rs`, `mcp_adapter.rs` | ~12 blocks | Reviewer cognitive load, shadowing risk |
| Z3 wrapped around a single boolean, not used as a solver | `legal.rs:198-228` | ~30 lines | Architecture communicates "Z3 solver" but code is "Z3 formalizer wrapper" |
| `Verb` trait lives alongside raw `Box<dyn LedgerOperation>` dispatch | `pipeline.rs:250`, `ledger_ops.rs:154` | ~15 lines | Two dispatch mechanisms for the same concept |
| `EvidenceBuilder` takes `&mut EvidenceGraph`, composes inline | `arc-kit-au/src/builder.rs:18` | ~317 lines | Works but hides graph mutation; no compile-time chain safety |
| `PipelineState<S>` typestate uses `PhantomData` correctly but no `StageResult<S>` | `pipeline.rs:32-58` | ~26 lines | Confidence/evidence carry-forward is runtime-checked, not type-checked |
| `#[serde(skip)]` on petgraph `DiGraph` means every deserialize rebuilds indexes | `arc-kit-au/src/graph.rs:48-55` | ~8 lines | Runtime cost on every state restore |
| `ToolError` is flat `InvalidInput(String) | Internal(String)` | `ledgerr-mcp/lib.rs:346-351` | ~6 lines | Loses type info from 19 upstream error enums |
| `ChatError` manually duplicates `AgentRuntimeError` variants without `#[from]` | `chat.rs:99-118` | ~20 lines | Maintenance burden; drift risk |
| 0 uses of `inspect()`, `map_or`, `GAT`, `#[doc(alias)]`, `#[non_exhaustive]` | Codebase-wide | — | Missed ergonomic and documentation patterns |

---

## 2. Scope (MECE by Layer)

### 2.1 Language & Edition Layer
- Edition 2024 migration across all crates
- `unsafe` block enforcement (edition requirement)
- `impl Trait` in RPIT for associated types
- MSRV declaration `1.85`, `rust-toolchain.toml` pin

### 2.2 Standard Library API Layer (1.89–1.94)
- `#[expect]` replaces `#[allow]` for lint-proof suppressions
- `{integer}::strict_*` ops in `ledger-core` financial math
- `LazyLock::get` / `LazyLock::force_mut` for once-init patterns
- `Duration::from_mins` / `from_hours` for readable timeouts
- `Path::file_prefix` for source-filename parsing
- `core::array::repeat` for constant array building

### 2.3 Idiom Cleanup Layer
- `let_chains` for nested if-let flattening
- `inspect()` for side-effect logging in pipeline chains
- `is_none_or()` / `map_or()` / `map_or_else()` for option predicates
- `.unwrap()` → `.expect()` in all non-test `src/`
- `clippy::unwrap_used = "deny"` under `#[cfg(not(test))]`

### 2.4 Crate & Abstraction Layer — New Adoptions

Each crate recommendation below targets a specific architectural surface and answers the question: *what stable, popular crate makes this surface more expressive in Rust's type/trait/generic/lifetime system?*

---

## 3. Crate-by-Crate Analysis & Recommendations

### 3.1 `arc-kit-au` — Evidence Graph with Typed State Machine

**Current architecture (lines 48-55, `graph.rs`):**
```rust
pub struct EvidenceGraph {
    nodes: Vec<EvidenceNode>,
    edges: Vec<EvidenceEdge>,
    #[serde(skip)]
    node_index: HashMap<NodeId, NodeIndex>,
    #[serde(skip)]
    graph: DiGraph<EvidenceNode, EdgeType>,  // rebuilt on deserialize
}
```

**`EvidenceBuilder` (lines 18-31, `builder.rs`):**
```rust
pub struct EvidenceBuilder<'a> {
    graph: &'a mut EvidenceGraph,
}
pub fn ensure_document(&mut self, doc: SourceDoc) -> NodeId { ... }
pub fn ensure_classification(&mut self, cls: Classification) { ... }
```

**What's good:** Idempotent `ensure_*` operations, `tracing::warn!` instead of panic on duplicate edges, flat-Vec serialization works.
**What's possible:** The builder's `&mut EvidenceGraph` borrow is unchecked — you can add nodes in any order, skip required edges, or add a `WorkbookRow` before a `Transaction`. The `ProvenanceBadge` enum models 4 states, but nothing prevents illegal state sequences at compile time.

#### Recommendation 1: **`arc-kit-au` → Typestate with `frunk` or manual `PhantomData`**

Replace flat `EvidenceBuilder` with a **chain builder** that encodes graph state in the type:

```rust
// Type-state markers
pub struct NeedsSource;      // graph is empty
pub struct HasDocuments;     // source docs added
pub struct HasExtracted;     // rows extracted
pub struct HasTransactions;  // transactions committed
pub struct HasClassified;    // classifications added
pub struct HasExported;      // workbook rows written

pub struct EvidenceChain<S> {
    graph: EvidenceGraph,
    _state: PhantomData<S>,
}

// Methods only available at the correct state
impl EvidenceChain<NeedsSource> {
    pub fn ingest_document(self, doc: SourceDoc) -> EvidenceChain<HasDocuments> { ... }
}
impl EvidenceChain<HasDocuments> {
    pub fn extract_rows(self, rows: Vec<ExtractedRow>, doc: &NodeId)
        -> EvidenceChain<HasExtracted> { ... }
}
impl EvidenceChain<HasExtracted> {
    pub fn commit_transaction(self, tx: Transaction)
        -> EvidenceChain<HasTransactions> { ... }
}
impl EvidenceChain<HasTransactions> {
    pub fn classify(self, cls: Classification)
        -> EvidenceChain<HasClassified> { ... }
}
impl EvidenceChain<HasClassified> {
    pub fn export_to_workbook(self, row: WorkbookRow)
        -> EvidenceChain<HasExported> { ... }
}
```

**Why this crate pattern:** No new external crate needed — `PhantomData` is std. The pattern mirrors `PipelineState<S>` in `pipeline.rs` and gives the evidence graph the same compile-time safety as the pipeline. Makes the application's **provenance chain visible in the type signature** — one of its unique capabilities.

`frunk` (already a dev-dependency) provides `HList` for building heterogeneous lists as types, useful if EvidenceChain needs to encode which node types exist generically:
```rust
use frunk::HList;
type CompleteChain = HList!(SourceDoc, ExtractedRow, Transaction, Classification, WorkbookRow);
```
But for PRD-6 scope, manual `PhantomData` typestate is sufficient and more maintainable.

#### Recommendation 2: **Petgraph persistence via `bincode` or `rkyv`**

`#[serde(skip)]` on `DiGraph` means every deserialize calls `add_node()` N times and `add_edge()` M times. For graphs with 10,000+ nodes (plausible for multi-year tax records), this is O(N+M) work on every load.

```rust
// Current: serde_json roundtrip with index rebuild
#[derive(Serialize, Deserialize)]
pub struct EvidenceGraph {
    nodes: Vec<EvidenceNode>,
    edges: Vec<EvidenceEdge>,
    #[serde(skip)]
    graph: DiGraph<EvidenceNode, EdgeType>,  // rebuilt every deserialize
}

// Proposed: rkyv with zero-copy deserialize
// rkyv serializes the petgraph directly, including indices
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct EvidenceGraph {
    graph: DiGraph<EvidenceNode, EdgeType>,
    // Flat Vecs become derived fields, not serialized separately
}
```

`rkyv` 0.8.15 is already in the tech stack (per `AGENTS.md`). This eliminates the rebuild cost and simplifies the serialization contract.

---

### 3.2 `ledger-core/src/legal.rs` — Z3 from Wrapper to First-Class Solver

**Current architecture (lines 198-228):**
```rust
#[cfg(feature = "legal-z3")]
fn violation_result(&self, violation: bool, witness: &str) -> Z3Result {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    let violation = Bool::from_bool(&ctx, violation);  // encodes a boolean constant
    let result = sat_to_rule_result(
        solver.check_assumptions(&[violation]),
        witness,
    );
    result
}
```

**What's good:** Clear feature-gate separation. Fallback path without Z3 is identical behavior. Z3 is optional, not mandatory.
**What's possible:** Currently Z3 is a *formalizer*, not a *solver* — it confirms a violation that was already computed in Rust. The architecture communicates "Z3 constraint solver" but the code is "SAT wrapper around a pre-computed boolean." For PRD-6, the application should **use Z3 as an actual solver** for the constraints it already models.

#### Recommendation 3: **Encode tax rules as symbolic Z3 constraints**

Transform AU GST s38-190 and US Schedule C rules from `if/else` into Z3 `Bool` expressions with free variables, then let Z3 find *witness assignments*:

```rust
#[cfg(feature = "legal-z3")]
pub fn verify_au_gst_38_190_z3(facts: &TransactionFacts) -> Z3Result {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    // Free variables — let Z3 find the violation witness
    let is_travel = Bool::new_const(&ctx, "is_travel_related");
    let is_meal = Bool::new_const(&ctx, "is_meal_entertainment");
    let amount_exceeds = Bool::new_const(&ctx, "amount_exceeds_threshold");

    // AU GST Act s38-190: non-deductible if meal/entertainment with travel
    // AND amount > $300 per person
    let rule = is_meal.and(&[&is_travel, &amount_exceeds]).not();
    solver.assert(&rule);

    // facts become assumptions
    let facts_assumptions = &[
        Bool::from_bool(&ctx, facts.is_meal_entertainment),
        Bool::from_bool(&ctx, facts.is_travel_related),
        Bool::from_bool(&ctx, facts.amount > Decimal::from(300)),
    ];

    match solver.check_assumptions(facts_assumptions) {
        SatResult::Unsat => Z3Result::Satisfied,  // rule holds — no violation
        SatResult::Sat => Z3Result::Violated {
            witness: solver.get_model()  // Z3 tells us WHY
                .map(|m| format!("{m:?}"))
                .unwrap_or_default(),
        },
        SatResult::Unknown => Z3Result::Unknown,
    }
}
```

**Why this crate pattern:** Uses the existing `z3` crate as an actual constraint solver, not a boolean wrapper. The feature-gate pattern stays the same. The application's *Z3 capability* becomes real, not nominal. Models produce witness traces.

---

### 3.3 `ledger-core/src/pipeline.rs` — `Verb` Trait as `enum_dispatch` or Typed Executor

**Current architecture (lines 250-260):**
```rust
pub trait Verb: Send + Sync + 'static {
    type Input: Serialize + DeserializeOwned;
    type Output: Serialize + DeserializeOwned;
    fn name(&self) -> &'static str;
    fn reversibility(&self) -> Reversibility;
    fn access(&self) -> AccessCriteria;
    fn execute(&self, input: Self::Input) -> (Vec<Issue>, Self::Output);
}
```

**Alongside (lines 154-162, `ledger_ops.rs`):**
```rust
pub trait LedgerOperation: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn is_idempotent(&self) -> bool { false }
    fn execute(&self, ctx: &OperationContext) -> Result<OperationResult, LedgerOpError>;
}
```

**What's good:** Two trait hierarchies for two concerns (pipeline verbs vs. ledger operations). Associated types on `Verb` enforce Input/Output typing.
**What's possible:** Two trait hierarchies is the right call, but the dispatch (in practice) overlaps. `enum_dispatch` would replace `Box<dyn Verb>` with a monomorphized enum, eliminating vtable overhead and enabling inlining.

#### Recommendation 4: **`enum_dispatch` for Verb and LedgerOperation**

```rust
#[enum_dispatch]
pub trait Verb {
    fn name(&self) -> &'static str;
    fn reversibility(&self) -> Reversibility;
    fn access(&self) -> AccessCriteria;
}

#[enum_dispatch(Verb)]
pub enum VerbImpl {
    ClassifyVerb(ClassifyVerbImpl),
    ValidateVerb(ValidateVerbImpl),
    ReconcileVerb(ReconcileVerbImpl),
    CommitVerb(CommitVerbImpl),
}
```

**Why this crate:** `enum_dispatch` is stable, popular (1.5k GitHub stars, `crates.io` 800k+ downloads), and zero-cost — it converts trait dispatch into a `match` over the enum discriminant. No `Box<dyn Verb>` allocation, no vtable lookup. Type erasure becomes type enumeration.

---

### 3.4 `ledger-core/src/verify.rs` — `MultiModelVerifier` with `trait_variant`

**Current architecture (lines 105-167):**
```rust
pub trait ModelClient: Send + Sync {
    fn complete(&self, prompt: &str, max_tokens: usize) -> anyhow::Result<String>;
    fn extract<T: DeserializeOwned>(&self, prompt: &str) -> anyhow::Result<T>;
}

pub struct MultiModelVerifier<C: ModelClient> {
    proposer: C,
    reviewer: C,
    config: MultiModelConfig,
}
```

**What's good:** Generic over `C: ModelClient`, both proposer and reviewer can be different types. `extract` uses `DeserializeOwned`.
**What's possible:** Currently sync. If async is needed later, `#[trait_variant]` (stable since Rust 1.80) generates an async variant automatically.

#### Recommendation 5: **`trait_variant` for sync/async duality**

```rust
#[trait_variant(pub trait ModelClient: Send + Sync)]
impl ModelClient {
    async fn complete(&self, prompt: &str, max_tokens: usize) -> anyhow::Result<String>;
    async fn extract<T: DeserializeOwned>(&self, prompt: &str) -> anyhow::Result<T>;
}
```

**Why this crate pattern:** No external crate — `#[trait_variant]` is std since 1.80. The generated code creates two traits: `ModelClient` (sync) and `ModelClientAsync` (async). The application already has both sync paths (Rhai classify, deterministic fallback) and async paths (Rig agent runtime). `trait_variant` lets them share the same trait definition while being usable from both contexts.

---

### 3.5 `ledgerr-mcp/src/lib.rs` — `ToolError` with `snafu` or `thiserror` Domain Typing

**Current architecture (lines 346-351):**
```rust
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Internal error: {0}")]
    Internal(String),
}
```

**Alongside (lines 99-118, `chat.rs`):**
```rust
// ChatError manually duplicates AgentRuntimeError variants WITHOUT #[from]
pub enum ChatError {
    Runtime(std::io::Error),           // no #[from]
    Rig(CompletionError),              // no #[from]
    Parse(serde_json::Error),          // no #[from]
}
```

**What's good:** Simple, two-variant `ToolError`. Codebase has 19 `thiserror` enums total.
**What's possible:** `ChatError` is a manual copy of `AgentRuntimeError` with `#[from]` deliberately omitted. This is a maintenance smell — any new `AgentRuntimeError` variant requires a manual `ChatError` update. Also, `ToolError`'s `String` fields lose the structured data from upstream errors.

#### Recommendation 6: **Rich `ToolError` variants with error source chain**

```rust
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Invalid input: {detail}")]
    InvalidInput { detail: String, source: Option<Box<dyn std::error::Error + Send>> },
    #[error("Internal error: {detail}")]
    Internal { detail: String, source: Option<Box<dyn std::error::Error + Send>> },
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

// Manual From impls preserve upstream type info
impl From<FilenameError> for ToolError {
    fn from(e: FilenameError) -> Self {
        ToolError::InvalidInput {
            detail: e.to_string(),
            source: Some(Box::new(e)),
        }
    }
}
```

Alternatively, use **`snafu`** for error typing with `#[context]` attributes that preserve source chains without manual `From` impls:

```rust
use snafu::{Snafu, ResultExt, Whatever};

#[derive(Debug, Snafu)]
pub enum ToolError {
    #[snafu(display("Invalid input: {detail}"))]
    InvalidInput { detail: String, source: FilenameError },
    #[snafu(display("Internal error: {detail}"))]
    Internal { detail: String, source: Box<dyn std::error::Error + Send> },
}
```

**Why this crate:** `snafu` is the standard alternative to `thiserror` for large error enums with multiple upstream sources and structured context. Its `ResultExt` trait provides `.context()` / `.with_context()` that capture file paths, input values, etc., directly at the call site without extra `match` arms. The primary gain for PRD-6: `ChatError` no longer needs to manually duplicate `AgentRuntimeError`.

---

### 3.6 `ledger-core/src/rule_registry.rs` — `SemanticRuleSelector` with Real Embeddings

**Current architecture (lines 181-197):**
```rust
pub trait SemanticRuleSelector {
    fn select_rules_semantic(&self, tx: &SampleTransaction, top_k: usize) -> Vec<PathBuf>;
    fn build_embedding_index(&mut self) -> Result<(), RuleRegistryError>;
}

// Implementation: lexical Jaccard similarity fallback (lines 205-275)
fn lexical_similarity(query: &str, candidate: &str) -> f64 {
    let q_tokens: BTreeSet<_> = semantic_tokens(query);
    let c_tokens: BTreeSet<_> = semantic_tokens(candidate);
    let intersection = q_tokens.intersection(&c_tokens).count();
    let union = q_tokens.union(&c_tokens).count();
    if union == 0 { 0.0 } else { intersection as f64 / union as f64 }
}
```

**What's good:** Clear trait boundary. Jaccard fallback is correct and deterministic. The semantic selection is explicitly opt-in through `select_rules_semantic()`.
**What's possible:** Currently the "semantic" path is lexical-only. A `candle`-powered embedding backend would make the semantic path real.

#### Recommendation 7: **`candle` or `fastembed` for real embedding-based rule selection**

```rust
use candle_core::Device;
use candle_nn::Embedding;

pub struct CandleEmbeddingSelector {
    model: candle_transformers::models::bert::BertModel,
    device: Device,
    tokenizer: tokenizers::Tokenizer,
}

impl SemanticRuleSelector for CandleEmbeddingSelector {
    fn select_rules_semantic(&self, tx: &SampleTransaction, top_k: usize) -> Vec<PathBuf> {
        // 1. Encode tx description + account_id + amount into embedding
        // 2. Dot-product with pre-computed rule embeddings
        // 3. Return top_k rule paths
    }
}
```

**Why this crate:** `candle` is HuggingFace's minimal ML framework for Rust — no Python runtime, ONNX-compatible, CUDA/Metal/CPU. It makes the PRD-4 Phase 6 "semantic retrieval" requirement real without adding Python to the stack. The lexical Jaccard fallback stays as the `no-std` fallback; `candle` becomes an optional `#[cfg(feature = "candle-embeddings")]` backend.

---

### 3.7 `crates/ledgerr-host/src/agent_runtime.rs` — Rig with Structured Extraction via `serde_path_to_error`

**Current architecture (lines 218-241):**
```rust
pub enum AgentRuntimeError {
    #[error("Missing endpoint")]
    MissingEndpoint,         // manual check, not dereived from config
    #[error("Runtime error: {0}")]
    Runtime(#[from] std::io::Error),
    #[error("Rig error: {0}")]
    Rig(#[from] CompletionError),
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Invalid typed output: {0}")]
    InvalidTypedOutput(String),  // human-readable, no path info
}
```

**What's good:** Clear enum, correct `#[from]` usage. The `InvalidTypedOutput` variant carries structured validation feedback.
**What's possible:** When an LLM returns malformed JSON, the error says "invalid typed output: missing field 'confidence'" but doesn't tell you *where* in the response the problem occurred.

#### Recommendation 8: **`serde_path_to_error` for structured deserialization errors**

```rust
use serde_path_to_error as serde_path;

pub fn extract<T: DeserializeOwned>(&self, response: &str) -> Result<T, AgentRuntimeError> {
    let mut de = serde_json::Deserializer::from_str(response);
    serde_path::deserialize(&mut de)
        .map_err(|e| AgentRuntimeError::InvalidTypedOutput(format!(
            "at path '{}': {}",
            e.path().to_string(),
            e.inner()
        )))
}
```

**Why this crate:** `serde_path_to_error` wraps any serde `Deserializer` and annotates errors with the JSON path (`$.transactions[3].category`). This turns "parse error" into actionable debugging info — critical when an LLM generates the JSON. The crate is tiny (~200 lines), stable, and universally compatible with serde.

---

### 3.8 Cross-Cutting: `typed-builder` for Builder Pattern Elimination

**Builders exist in:** `pipeline.rs:365-403`, `arc-kit-au/src/builder.rs`, and ad-hoc throughout tests.

#### Recommendation 9: **`typed-builder` for all struct construction with >3 fields**

```rust
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Classification {
    pub tx_id: String,
    pub category: String,
    #[builder(default, setter(strip_option))]
    pub sub_category: Option<String>,
    #[builder(default = 0.8)]
    pub confidence: f64,
    #[builder(default)]
    pub reason: Option<String>,
}

// Usage — compile-time field name checking
let cls = Classification::builder()
    .tx_id("tx_123".into())
    .category("Meals".into())
    .confidence(0.95)
    .build();
```

**Why this crate:** `typed-builder` is stable, popular (3k GitHub stars, 50M+ downloads), and generates builders with named setters at compile time — no `...builder.field("key", val)` runtime errors. It eliminates boilerplate while keeping the typestate-safety of named field construction.

---

### 3.9 Cross-Cutting: `derive_more` for Trait Boilerplate

**Current:** Node types manually implement `Display`, `From`, `AsRef`, etc. across 448 lines in `node.rs`.

#### Recommendation 10: **`derive_more` for display, from, constructor, into**

```rust
use derive_more::{Display, From, Into, Constructor, AsRef};

#[derive(Debug, Clone, Serialize, Deserialize, Display, From, Into, Constructor)]
#[display("txn:{tx_id}:{category}")]
pub struct Classification {
    pub tx_id: String,
    pub category: String,
    pub sub_category: Option<String>,
    pub confidence: f64,
    pub actor: String,
    pub classified_at: DateTime<Utc>,
    pub note: Option<String>,
}
```

**Why this crate:** `derive_more` generates `Display`, `From`, `Into`, `AsRef`, `Deref`, `Mul`, `Add`, etc. via derive macros — eliminating 50-100 lines of manual `impl` blocks per crate. Already in the ecosystem since Rust 1.0; `derive_more` 2.0 supports all current derive macro patterns.

---

### 3.10 Cross-Cutting: `self_cell` / `ouroboros` for Self-Referential Graphs

**Current in `arc-kit-au/src/graph.rs`:** The `DiGraph` and `Vec<EvidenceNode>` fields are kept separate to avoid self-referential struct issues.

#### Recommendation 11: **`self_cell` for self-referential evidence graph serializer**

```rust
use self_cell::self_cell;

self_cell!(
    pub struct EvidenceGraphCell {
        owner: Vec<EvidenceNode>,
        #[covariant]
        dependent: IndexSet,  // or any type that borrows from owner
    }
);
```

**Why this crate:** `self_cell` enables safe self-referential structs without `unsafe` or pin-projection. It lets the petgraph `DiGraph` borrow internal vecs directly, eliminating the `#[serde(skip)]` → rebuild dance. The graph keeps its canonical flat-nodes representation for serialization while giving petgraph direct access to node data for traversal. Limited to 0.8.x stable.

---

## 4. Summary: Crate Adoption + Edition Migration (Combined Inventory)

| # | Crate / Pattern | Version | Surface | Why |
|---|---|---|---|---|
| 1 | **Typestate `PhantomData` chain** (no crate) | 1.0 | `arc-kit-au` | Compile-time evidence provenance — makes application's unique chain safety visible |
| 2 | **`rkyv` for petgraph persistence** | 0.8.15 | `arc-kit-au` | Zero-copy graph load, eliminates O(N+M) rebuild |
| 3 | **`z3` symbolic constraints** (exists, refactored) | 0.8 | `ledger-core/legal.rs` | Real Z3 solver, not boolean wrapper. Makes Z3 capability genuine |
| 4 | **`enum_dispatch`** | 0.14 | `ledger-core/pipeline.rs` | Zero-cost verb dispatch, replaces `Box<dyn Verb>` |
| 5 | **`#[trait_variant]`** (stdlib) | 1.80 | `ledger-core/verify.rs` | Sync/async duality for `ModelClient` |
| 6 | **`snafu` or rich `thiserror`** | 0.8 | `ledgerr-mcp`, `ledgerr-host` | Preserved error source chains, eliminates `ChatError` duplication |
| 7 | **`candle` (optional)** | 0.9 | `ledger-core/rule_registry.rs` | Real embedding-based rule selection per PRD-4 Phase 6 |
| 8 | **`serde_path_to_error`** | 0.1 | `ledgerr-host/agent_runtime.rs` | LLM JSON parse errors with field path traces |
| 9 | **`typed-builder`** | 0.20 | All crates | Named compile-time builders for 3+ field structs |
| 10 | **`derive_more`** | 2.0 | All crates | 50-100 lines of Display/From/Into boilerplate eliminated |
| 11 | **`self_cell`** | 0.8 | `arc-kit-au/graph.rs` | Self-referential graph avoids index-rebuild cost |
| 12 | **Edition 2024** | — | All crates | `unsafe` enforcement, `impl Trait` RPIT, macro hygiene |

---

## 5. MECE Categorization (Layer View)

```
PRD-6 (Comprehensive)
├── Language / Edition
│   ├── Edition 2024 migration (§4 item 12)
│   └── MSRV + rust-toolchain.toml pin
├── Std API Adoption
│   ├── #[expect], LazyLock, Duration, Path, strict_*, array_windows
│   ├── let_chains, inspect(), is_none_or(), map_or()
│   └── clippy unwrap_used = deny
├── Type System Refinement
│   ├── Typestate EvidenceChain (§4 item 1)
│   ├── enum_dispatch Verb → zero-cost dispatch (§4 item 4)
│   ├── #[trait_variant] ModelClient sync/async (§4 item 5)
│   ├── typed-builder for struct construction (§4 item 9)
│   └── derive_more for trait boilerplate (§4 item 10)
├── Safety & Correctness
│   ├── Z3 symbolic constraint solver (§4 item 3)
│   ├── rkyv zero-copy graph (§4 item 2)
│   ├── self_cell safe self-ref graph (§4 item 11)
│   └── snafu/rich error with source chains (§4 item 6)
├── AI Integration
│   ├── candle embedding selector (§4 item 7)
│   └── serde_path_to_error LLM JSON (§4 item 8)
└── Test & Tooling Infrastructure (§6)
    ├── T1  cargo nextest (parallel CI test runner)
    ├── T2  proptest (property-based financial tests)
    ├── T3  insta (snapshot testing for serde outputs)
    ├── T4  rstest (parameterized integration tests)
    ├── T5  criterion (graph/classify/solve benchmarks)
    ├── T6  cargo-fuzz (Rhai parser input fuzzing)
    ├── T7  iai/hyperfine (build regression gate)
    ├── T8  cargo profile overrides (.cargo/config.toml)
    ├── T9  CI nextest partitioning (3-way sharding)
    ├── T10 Justfile guard dedup (shared ensure-* recipes)
    ├── T11 cargo-binstall fast-installs in CI
    └── T12 Remove dead deps fdg-sim, femtovg
```

---

## 6. Test & Tooling Infrastructure (New)

The new devtools (`rg`, `fd`, `bat`, `hyperfine`, `cargo-binstall`) expose gaps in the current test infrastructure that PRD-6 should close alongside language modernization.

### 6.1 Current State

| Capability | Status | Pain Point |
|---|---|---|
| Test runner | `cargo test` (serial) | ~117 test functions; `cargo test --workspace --all-targets --all-features` exceeds 5 minutes |
| Test isolation | None — all tests share process | No sandbox for filesystem I/O tests; leaked state between integration tests |
| Property testing | Zero — no `proptest`, `quickcheck`, or `rstest` | Edge cases in Z3/cassowary constraints are manually enumerated, not generated |
| Snapshot testing | Zero — no `insta` or `expect_test` | 46 serde JSON round-trips in tests are checked by hand, not by snapshot diff |
| Benchmark harness | Zero — no `criterion` or `iai` | Can't measure regressions in petgraph traversal, Rhai classification, or Z3 solving |
| Fuzz testing | Zero — no `cargo-fuzz` or `afl` | `parse_source()` in the Rhai parser has no generated-input coverage |
| CI test parallelism | `cargo test` (single job) | No `cargo nextest` — CI wastes ~50% of runner time on serial test execution |
| Cargo profile config | None — no `.cargo/config.toml` profiles | Dev builds are unoptimized; no `codegen-units = 1` for release benchmarks |
| `just` tooling | Ad-hoc cargo installs in recipes | `docgen`, `docserve`, `docgen-check` each duplicate the `cargo install` guard logic |

### 6.2 New Test Capabilities (by PRD-6 Phase)

#### Phase 0: Test Infrastructure

| # | Change | Tool | Effort | Value |
|---|---|---|---|---|
| T1 | **Adopt `cargo nextest`** as the primary test runner | `cargo nextest` (binstall) | 1 hour | Parallel test execution, per-test timing, JUnit XML output. Typical 3-5x speedup. CI job `cargo test` → `cargo nextest run` |
| T2 | **Add `proptest` to `ledger-core`** for property-based tests | `proptest` 1.x | 1 day | Generated transaction amounts, dates, descriptions → verify `deterministic_tx_id()` is collision-free and stable. Generate Z3 constraint combinations → verify solver doesn't panic |
| T3 | **Add `insta` snapshot testing** for serde JSON outputs | `insta` 1.x | 1 day | `OntologySnapshot::to_pretty_json_stable()`, `EvidenceGraph` serialization, `PipelineState` round-trips — one `insta::assert_json_snapshot!()` replaces 10 manual assertions |
| T4 | **Add `rstest` for parameterized tests** | `rstest` 0.23 | 0.5 day | Replace manual test-data-fn duplication in `mcp_adapter_contract.rs`, `phase4_audit_integrity.rs` with `#[rstest]` + `#[case]` |

#### Phase 1: Benchmark & Fuzz

| # | Change | Tool | Effort | Value |
|---|---|---|---|---|
| T5 | **Add `criterion` benchmarks** for performance-critical paths | `criterion` 0.5 | 2 days | Petgraph traversal (arc-kit-au), Rhai classification (rule_registry), Z3 solving (legal.rs). Baseline + regression detection in CI |
| T6 | **Add `cargo-fuzz` target** for Rhai parser | `cargo-fuzz` | 1 day | `mdbook-rhai-mermaid/src/parser.rs` takes untrusted user input (Rhai DSL). Fuzzing with generated strings finds panic/edge-case paths the manual tests miss |
| T7 | **Add `iai` or `hyperfine` regression gate** on CI | `iai` 0.1 / `hyperfine` | 0.5 day | Measure `cargo build` time for `ledgerr-mcp` — catch unintentional compilation regressions from `derive_more` or `typed-builder` macro expansion |

#### Phase 2: CI & Tooling

| # | Change | Tool/Script | Effort | Value |
|---|---|---|---|---|
| T8 | **Add `cargo profile` overrides** in `.cargo/config.toml` | — | 0.5 day | `[profile.dev] codegen-units = 256` for fast iteration (current default). `[profile.bench]` with `lto = "fat"`, `codegen-units = 1` for reliable benchmarks |
| T9 | **Update CI `test-and-build` job** to use `cargo nextest` | CI yml | 1 hour | Swap `cargo test` for `cargo nextest run --workspace --all-targets --all-features`. Add `--partition count:1/3` for 3-way sharding when test count grows |
| T10 | **Factor `just install-{tool}` guards** into a shared recipe | Justfile | 0.5 day | Replace the 6 duplicated `@if [ ! -x ~/.cargo/bin/mdbook ]; then ...` blocks with `just ensure-mdbook`, `just ensure-mcp-parser` |
| T11 | **Add `cargo-binstall` powered fast-installs** to CI | Justfile + CI | 1 hour | CI currently compiles `clippy-sarif`, `mdbook`, `mdbook-mermaid`, `mdbook-admonish` from source (~5 min each). `cargo binstall` installs them in seconds |
| T12 | **Remove dead deps `fdg-sim`, `femtovg`** | `ledger-core/Cargo.toml` | 5 min | Unblocks WASM compilation, reduces compile time, removes false-positive `cargo audit` surface |

### 6.3 Tooling Metrics

| Metric | Current | Target | How |
|---|---|---|---|
| CI test time | ~5 min | <2 min | `cargo nextest` parallelism + `cargo binstall` tooling |
| Test assertions per integration test | ~15 manual | ~30 (10 snapshot + 10 prop + 10 param) | `insta` snapshots + `proptest` generators |
| Code paths tested by generation | 0 | 500+ per property test | `proptest` — transaction hash, Z3 constraint, filename validation |
| Rhai parser fuzz coverage | 0 | 10,000+ inputs | `cargo-fuzz` on `parser::parse()` |
| Benchmark baselines | 0 | 3 (graph, classify, solve) | `criterion` in `benches/` |
| Build time change detection | None | CI fails if >10% regression | `iai` or `hyperfine` gate |
| `just` recipe duplication | 6 copies of the same guard | 0 (1 shared recipe) | `just ensure-mdbook` |

### 6.4 Implementation Order

```
Phase 0 (parallel with language changes)
├── T1  cargo nextest (CI swap)
├── T2  proptest for ledger-core
├── T3  insta for serde snapshots
├── T4  rstest for parameterization
├── T12 Remove dead deps fdg-sim, femtovg
└── T11 cargo-binstall in CI

Phase 1 (after T1 is stable)
├── T5  criterion benchmarks
├── T6  cargo-fuzz for Rhai parser
└── T7  iai/hyperfine regrssion gate

Phase 2 (ongoing)
├── T8  cargo profile config
├── T9  CI nextest partitioning
└── T10 Justfile guard dedup
```

---

## 7. Implementation Plan

### Phase 0: Std API + Idiom + Test Infra (1 sprint, no risk)
1. `#[expect]` migration, `inspect()`, `let_chains`, `is_none_or()`, `map_or()`
2. `.unwrap()` → `.expect()` in non-test `src/`
3. `clippy::unwrap_used = "deny"` gate
4. `rust-toolchain.toml` + `rust-version`
5. `Duration::from_mins`, `Path::file_prefix`, `strict_*` in financial math
6. `serde_path_to_error` in `agent_runtime.rs`
7. **T1-T4, T11-T12** — `cargo nextest`, `proptest`, `insta`, `rstest`, `cargo-binstall` CI, dead dep removal

### Phase 1: Trait + Type Refinement + Benchmarks (1 sprint)
1. `derive_more` across all crates (Display, From, Into, Constructor)
2. `typed-builder` for classification, proposal, evidence node structs
3. `enum_dispatch` for `Verb` trait → `VerbImpl` enum
4. `#[trait_variant]` on `ModelClient`
5. `snafu` or rich `ToolError` + eliminate `ChatError` duplication
6. **T5-T7** — `criterion` benchmarks, `cargo-fuzz`, `iai` regression gate

### Phase 2: Core Architecture + CI (2 sprints)
1. **Typestate `EvidenceChain<S>`** — replace `EvidenceBuilder` with compile-time chain
2. **Self-referential graph** — `self_cell` + `rkyv` for zero-copy petgraph persistence
3. **Refactor Z3** — from boolean wrapper to symbolic constraint solver with `check_assumptions` on free variables
4. Edition 2024 migration per crate
5. **T8-T10** — Cargo profiles, CI nextest partitioning, Justfile guard dedup

### Phase 3: Optional AI Embeddings (1 sprint, gated)
1. `candle` embedding selector behind `#[cfg(feature = "candle-embeddings")]`
2. Integration test comparing lexical vs. semantic rule selection on real transaction descriptions

---

## 8. Success Metrics

### 8.1 Language Modernization

| Metric | Current | Target | Where |
|---|---|---|---|
| `.unwrap()` in `src/` | ~80 | 0 | All `crates/*/src/` |
| `if let` nesting depth | 4 | 2 | `parser.rs`, `trace.rs`, `mcp_adapter.rs` |
| Builder boilerplate | ~200 lines manual | ~30 lines + derives | All crates |
| `Display`/`From`/`Into` manual impls | ~60 lines | 0 | `node.rs`, error types |
| Z3 constraint variables | 0 (boolean input) | 3-5 symbolic vars | `legal.rs` |
| Graph deserialize cost | O(N+M) rebuild | O(1) zero-copy | `arc-kit-au/src/graph.rs` |
| Verb dispatch cost | vtable + heap | inlined match | `pipeline.rs` |
| ChatError-other duplication | full manual copy | 0 (shared source) | `chat.rs` |
| `[allow]` count | 6 | 0 (all `[expect]`) | Codebase-wide |
| Edition | 2021 | 2024 | All Cargo.toml |

### 8.2 Test & Tooling

| Metric | Current | Target | How |
|---|---|---|---|
| CI test time | ~5 min | <2 min | `cargo nextest` parallelism + `cargo binstall` tooling |
| Test runner | `cargo test` (serial) | `cargo nextest` (parallel) | CI yml swap |
| Property tests | 0 | 3 modules (hash, z3, filename) | `proptest` in `ledger-core/tests/` |
| Snapshot tests | 0 | 10+ snapshots | `insta` for serde JSON output |
| Parameterized tests | 0 | 20+ `#[case]` inputs | `rstest` in MCP adapter, audit tests |
| Benchmark suites | 0 | 3 (graph, classify, solve) | `criterion` in `benches/` |
| Fuzz targets | 0 | 1 (Rhai parser) | `cargo-fuzz` on `parser::parse()` |
| Build regression gate | None | CI fails on >10% regression | `iai` or `hyperfine` |
| Dead deps blocking WASM | `fdg-sim`, `femtovg` | 0 | Removed from `ledger-core/Cargo.toml` |
| `just` recipe duplication | 6 copies | 0 (1 shared recipe) | `just ensure-mdbook` |

---

## 9. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Typestate chain breaks dynamic builder pattern tests | Medium | Medium | Keep `EvidenceBuilder` as deprecated wrapper around `EvidenceChain::<HasTransactions>` — tests migrate gradually |
| `self_cell` API surface changes in 0.9 | Low | High | Pin `self_cell = "=0.8.x"`; the crate is mature (0.8 was stable for 2+ years) |
| `candle` GPU-backend complexity on Windows | Medium | Low | Gate behind `cfg(feature = "candle-embeddings")`; keep Jaccard fallback as default |
| Z3 symbolic refactor changes test behavior | Low | Medium | Wrapped in `#[cfg(test)]` property tests: symbolic solver output must match if/else output for all known facts |
| `enum_dispatch` incompatible with associated types on `Verb` | Medium | High | Test with `cargo check` before committing; fallback: keep `Box<dyn Verb>` for `Verb` only, `enum_dispatch` for `LedgerOperation` |
| Edition 2024 breaks Slint macro output | Medium | High | Test `cargo build -p ledgerr-tauri` after each phase; keep edition-2021 on `ledgerr-tauri` if needed |
| `cargo nextest` requires nightly `dev` proc-macro support | Low | Medium | `cargo nextest` works on stable; test with `cargo nextest run` before CI swap |
| `proptest` shrinks find latent bug in financial hash | Low | High | Feature — this is the *goal*; property tests are designed to find these |
| `cargo-fuzz` catches panic in parser but fix requires parser refactor | Medium | Low | Pure gain — finding panics in untrusted input parsing is the purpose |

---

## 10. Appendix D: WASM as a Target — Good-Faith Critical Review

### D.1 The Question

*"Would deeper integration of WASM as a target benefit or complicate the application's internal shape and capability?"*

Short answer: **WASM as a first-class target would complicate the codebase significantly for marginal current benefit, but a scoped WASM compilation of the pure-data crates (arc-kit-au, ledger-core computation) would improve the Rhai docs live-editor and enable in-browser evidence graph visualization without reshaping the application's architecture.**

### D.2 Compatibility Matrix (Per Crate)

| Crate | WASM Status | Key Blockers | Lines of `std::fs` |
|---|---|---|---|
| **arc-kit-au** | ✅ **Works-as-is** (core types) | `store.rs` uses `std::fs` (7 sites), separable | ~7 |
| **ledger-core** | ⚠️ **Needs feature-gating** | `fdg-sim`/`femtovg` (dead deps), `z3` (C FFI, already gated), 15 `std::fs` sites | ~15 |
| **ledgerr-mcp** | 🔴 **Incompatible** | `sysinfo`, `PathBuf`-addressed I/O throughout, 20+ `std::fs` sites | ~20 |
| **ledgerr-host** | 🔴 **Incompatible** | Raw `TcpListener` server, `std::process` (5+ spawns), `tokio full`, Slint/windows-sys/tray-icon | ~20 |
| **ledgerr-xero** | ⚠️ **Needs async migration** | `reqwest::blocking` + local OAuth2 redirect server | ~3 |
| **ledgerr-llm** | ⚠️ **Blocking reqwest** | Same blocking-in-WASM issue | 0 |
| **ledgerr-tauri** | 🔴 **Incompatible** | Tauri is a native desktop framework | ~1 |
| **mdbook-rhai-mermaid** | ✅ **Works-as-is** (parser) | `std::process::exit` in binary only | 0 |

### D.3 I/O Surface Summary (source only, non-test)

| Capability | Occurrences | WASM Impact |
|---|---|---|
| `std::fs::*` | ~103 total references | Core data model is `PathBuf`-addressed throughout `ledgerr-mcp` |
| `std::process::Command` | ~29 total (16 `Command::new`) | Process spawning for foundry, browser, PowerShell, host-window |
| `std::net::TcpListener/TcpStream` | 1 file | Raw HTTP server in `internal_openai.rs` — fundamental to host architecture |
| `std::thread` | 2 files (internal_openai, local_llm) | Blocking I/O threads; WASM needs dedicated workers |
| `tokio::runtime` | 2 occurrences | `tokio full` includes `process`, `net`, `rt-multi-thread` — WASM-incompatible |
| **Total platform-bound I/O** | **~136 references** | Application is fundamentally a local-first filesystem-native tool |

### D.4 Where WASM Helps — The Live Editor Use Case

The strongest WASM opportunity is **compiling the Rhai parser and `arc-kit-au` evidence graph to WASM for the browser-based docs live editor.**

Current state: `book/theme/rhai-live-core.js` (548 lines) is a full JS reimplementation of the Rust Rhai→Mermaid parser. It duplicates:
- `mdbook-rhai-mermaid/src/parser.rs` (661 lines) — Rhai DSL parsing
- `mdbook-rhai-mermaid/src/emitter.rs` (119 lines) — Mermaid generation
- `mdbook-rhai-mermaid/src/graph.rs` — graph construction

This means every parser bug fix or feature addition must be applied in **two codebases** (Rust and JS).

WASM path: Compile `arc-kit-au`'s evidence graph types + `ledger-core`'s petgraph analysis + the `mdbook-rhai-mermaid` parser to WASM via `wasm-pack` + `wasm-bindgen`. The browser loads a single `.wasm` blob instead of the 548-line JS reimplementation.

### D.5 Where WASM Does NOT Help (and Would Cause Harm)

| Attempt | Why It Hurts |
|---|---|
| WASM-compile the MCP server | Requires an abstract storage backend trait across 20+ `std::fs` sites, async refactoring of all synchronous I/O, removing `sysinfo`, removing `PathBuf` as canonical address. This is a multi-month rewrite that makes the codebase *less* idiomatic by adding abstraction layers where none are needed. |
| WASM-compile the host tray/window | The host runs a raw TCP server, spawns processes, renders Slint windows, sets system tray icons. Every one of these is fundamentally native-OS. WASM adds nothing. |
| WASM-compile Tauri | Tauri's backend is native Rust; the webview is HTML/JS. Compiling the Rust backend to WASM is architecturally backward — Tauri's value *is* its native capability. |
| Full `wasm-pack --target web` on `ledger-core` | Would force feature-gating `z3`, `xattr`, `fdg-sim`, `femtovg`. The dead deps (`fdg-sim`, `femtovg`) are the first blockers — but they should be removed or made optional regardless of WASM. |

### D.6 Effort Estimate

| Scope | Effort | What It Enables |
|---|---|---|
| Remove dead deps `fdg-sim`, `femtovg` from `ledger-core` | 1 hour | Unblocks WASM compilation of `ledger-core` computation |
| Compile `arc-kit-au` graph types to WASM | 2-3 days | In-browser evidence graph construction and traversal (no `store.rs` — just `EvidenceGraph`, `EvidenceBuilder`, `EvidenceTracer`, `ProvenanceScanner`) |
| Compile `mdbook-rhai-mermaid` parser to WASM | 2-3 days | Replace the JS-only reimplementation in `rhai-live-core.js` with identical Rust-generated parser output |
| `wasm-bindgen` bridge for `SemanticRuleSelector` (rule registry classify) | 3-5 days | In-browser transaction classification demo (pure computation, no I/O) |
| Abstract storage backend + async refactor for MCP server | 4-8 weeks | WASM-compilable MCP server. **Not recommended** until there is a concrete browser-hosted product requirement. |

### D.7 Recommendation

**Do not adopt WASM as a first-class target in PRD-6.**

Instead, take two bounded actions:

1. **Remove dead dependencies `fdg-sim` and `femtovg` from `ledger-core/Cargo.toml` immediately.** They block WASM compilation for zero benefit. This is a 5-minute fix that pays forward.

2. **Add a `wasm-scope` tracking issue** (not a PRD milestone) for compiling `arc-kit-au` + the Rhai parser to WASM for the docs live editor. The work is well-understood (pure Rust, no I/O) and produces a tangible benefit: eliminating the JS/Rust parser duplication. If stakeholders prioritize the browser experience over desktop, this becomes a `wasm-pack --target web` project that can ship independently.

**Rationale (TRIZ):** The application's core value proposition — local-first, filesystem-native, CPA-auditable bookkeeping — is *antithetical* to WASM's sandboxed, filesystemless execution model. The application's I/O surface (103 `std::fs` references, 29 `std::process` references) proves that the codebase is a native tool, not a web tool. Forcing WASM compatibility would add abstraction layers (trait-based storage backends, async wrappers around sync I/O) that make the code *less* readable and *less* idiomatic for zero product gain.

**TRIZ principle applied:** *Segmentation* — separate the pure-computation layers from the I/O layers, but do NOT cross-contaminate the I/O layers with WASM compatibility shims. Keep `arc-kit-au` and `ledger-core`'s pure computation WASM-compilable (they already mostly are); leave `ledgerr-mcp` and `ledgerr-host` as native-only. This is the same feature-gate pattern already used for `z3`, `xattr`, and `mistralrs-llm`.

### D.8 WASM-Specific Changes to Include in PRD-6 Phase 0

Two small changes that cost nothing and keep the WASM door open:

```toml
# Before (ledger-core/Cargo.toml):
fdg-sim = "0.9"        # force-directed graph — not used anywhere in src/
femtovg = "0.9"        # GPU vector renderer — not used anywhere in src/

# After:
# (removed — both were dead dependencies)
```

And in `arc-kit-au/src/store.rs`, make the persistence methods conditional:

```rust
// Before:
impl EvidenceStore {
    pub fn load(&self) -> Result<EvidenceGraph, StoreError> {
        let json = std::fs::read_to_string(&self.path)?;
        // ...
    }
}

// After:
impl EvidenceStore {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(&self) -> Result<EvidenceGraph, StoreError> {
        let json = std::fs::read_to_string(&self.path)?;
        // ...
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load(&self) -> Result<EvidenceGraph, StoreError> {
        Err(StoreError::Unsupported("filesystem I/O not available on WASM".to_string()))
    }
}
```

A single `#[cfg]` annotation in one file is all that's needed to make `arc-kit-au` WASM-compilable today. This is consistent with the existing `#[cfg(target_os = "linux")]` pattern on `xattr` in `fs_meta.rs`.
