# PRD-9: Isometric Pipeline Visualization — Object Layer Model, Lint Tests, and Animated Transform Path

**Status:** Active | **Priority:** P1 (Capability) | **Date:** 2026-05-02
**Depends on:** PRD-6 (type system), PRD-7 (synergy APIs), PRD-6-FUTURE (attestation concept)

---

## 1. Problem Statement

The `rhai-live-core.js` engine already renders a full isometric 3D scene from a Rhai DSL string: it solves `x,y,z` constraint layout, projects to screen via `isoProject((x-z)*0.866, (x+z)*0.5 - y)`, emits depth-faced polygons, animates node movement with `<animateTransform>`, and generates a `data:model/gltf+json` URI per node for future WebXR/Three.js pickup. The `SemanticRole` enum in the Rust parser mirrors the JS `SEMANTIC_CATEGORIES`.

What does not exist:

| Gap | Impact |
|---|---|
| No Rust `Vec3` / `IsoScene` / `iso_project()` — layout is 1D (x, width) only | Cannot generate or validate isometric scenes from Rust; pipeline visualization is JS-only |
| No `ZLayer` axis semantics — z is unused | Cannot express the verification stack (constraint layer, legal layer, kani layer, attest layer) as distinct depth planes |
| No `VisualizationSpec` trait — types have no declared visual identity | Cannot lint-test that a PRD-6 or PRD-7 type has a valid, parseable Rhai DSL representation |
| No Rhai DSL snippets for any PRD-6/7 object | The isometric view renders only hand-written doc diagrams; pipeline objects are invisible |
| No docgen chapter for the visualization model | Future developers cannot understand the layer semantics or contribute new visualizations |
| No `.tomllmd` dynamic section | Cannot use the autoresearch distillation pipeline to summarize or query visualization state |
| No animation model in Rust | `animation_styles()` emits flat CSS; no typed transform record that maps to `<animateTransform>` |
| No `manim`-style path recording | Cannot generate frame-by-frame transformation sequences for data processing animations |

---

## 2. Layer Model — `x, y, z` Semantic Axes

The isometric space has defined axes:

```
x  →  pipeline progress       (Ingested=0 … Committed=5)
y  ↑  confidence lift          (0.0 = floor, 1.0 = full lift; review lane sinks below floor)
z  ⊙  verification depth       (layer 0 … layer 5, see below)
```

### Z-Layer Stack

| z | Layer | Contents | Color family |
|---|---|---|---|
| 0 | Document | Raw PDFs, `ExtractedRow`, `TransactionInput` | slate `#334155` |
| 1 | Pipeline | `PipelineState<S>`, `MetaCtx`, `StageScore`, `Disposition` | blue `#1d4ed8` |
| 2 | Constraint | `VendorConstraintSet`, `ConstraintEvaluation`, `InvoiceVerification` | violet `#7c3aed` |
| 3 | Legal | `Z3Result`, `LegalRule`, `TransactionFacts`, `LegalSolver` | red `#b91c1c` |
| 4 | Formal proof | Kani harnesses, `KaniProofStatus` | teal `#0f766e` |
| 5 | Attestation | `#[attested]`, `InvariantEntry`, `InvariantLedger` | amber `#b45309` |

Layers are rendered back-to-front (z=0 deepest, z=5 frontmost). The shadow plane sits below z=0. The `CommitGate` decision node spans layers 1–3 (it aggregates pipeline confidence + constraint score + legal result).

---

## 3. Object Inventory — PRD-6 and PRD-7

Every object below requires:
1. A `VisualizationSpec` with `semantic_type`, `z_layer`, and `rhai_dsl`.
2. A lint test asserting the DSL parses to a non-empty graph.
3. A docgen entry in `book/src/iso-pipeline-objects.md`.

### PRD-6 Objects (z=0–2)

| Object | z | Semantic | Key DSL nodes |
|---|---|---|---|
| `PipelineState<Ingested>` | 1 | `ingest` | `document_received`, `check_constraints`, `validate` |
| `PipelineState<Validated>` | 1 | `validate` | `validate`, `verify_legal`, `classify` |
| `PipelineState<Classified>` | 1 | `classify` | `classify`, `reconcile` |
| `PipelineState<Reconciled>` | 1 | `reconcile` | `reconcile`, `evaluate_commit_gate` |
| `PipelineState<Committed>` | 1 | `storage` | `committed` |
| `PipelineState<NeedsReview>` | 1 | `human` | `needs_review`, `operator_queue` |
| `MetaCtx` | 1 | `data` | `stage_trace`, `accumulated_confidence`, `flags` |
| `Disposition` | 1 | `logic` | `match disposition => Unrecoverable -> halt`, etc. |
| `Issue` | 1 | `rule` | `issue_code`, `disposition`, `source` |
| `VendorConstraintSet` | 2 | `rule` | `evaluate_required`, `evaluate_strong`, `evaluate_medium` |
| `ConstraintEvaluation` | 2 | `intelligence` | `required_pass`, `strong_ratio`, `to_confidence`, `to_meta_flag` |
| `InvoiceConstraintSolver` | 2 | `rule` | `validate_arithmetic`, `validate_gst_rate`, `invoice_verify` |
| `InvoiceVerification` | 2 | `report` | `arithmetic_ok`, `gst_rate_ok`, `audit_note` |
| `EvidenceGraph` | 0 | `storage` | `add_node`, `add_edge`, `save_rkyv` |
| `EvidenceChain<S>` | 0 | `data` | `needs_source`, `has_documents`, `has_extracted`, `has_transactions`, `has_classified`, `has_exported` |

### PRD-7 Objects (z=2–3)

| Object | z | Semantic | Key DSL nodes |
|---|---|---|---|
| `Z3Result` | 3 | `logic` | `match z3_result => Satisfied -> pass`, `Violated -> halt`, `Unknown -> advisory` |
| `LegalRule` | 3 | `rule` | `rule_id`, `jurisdiction`, `formula`, `verify` |
| `TransactionFacts` | 3 | `data` | `vendor_jurisdiction`, `supply_type`, `tax_code`, `is_business_activity` |
| `LegalSolver` | 3 | `intelligence` | `verify`, `verify_all`, `confidence_from_rules` |
| `Jurisdiction` | 3 | `rule` | `match jurisdiction => US -> schedule_c`, `AU -> gst_38_190`, `UK -> empty` |
| `CommitGate` | 1–3 | `decision` | `match commit_gate => Approved -> commit`, `PendingOperator -> review`, `Blocked -> halt` |

---

## 4. `VisualizationSpec` Trait — Rust Interface

### 4.1 Core Types (`crates/ledger-core/src/iso.rs`)

```rust
/// Mirrors ISO_SETTINGS in rhai-live-core.js.
pub struct IsoSettings {
    pub level_gap: f32,     // 192.0 — x spacing between pipeline levels
    pub lane_gap: f32,      // 136.0 — z spacing between parallel lanes
    pub decision_lift: f32, // 34.0  — y lift for logic/decision nodes
    pub review_lift: f32,   // 18.0  — y lift for human review nodes
    pub commit_lift: f32,   // 12.0  — y lift for storage/commit nodes
    pub card_depth: f32,    // 20.0  — depth extrusion for isometric faces
    pub animation_ms: u32,  // 460   — transition duration
}

/// 3D position in the isometric model space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,  // pipeline progress (level * level_gap)
    pub y: f32,  // confidence lift
    pub z: f32,  // lane offset (z_layer * lane_gap or arm_index * lane_gap)
}

/// 2D screen position after isometric projection.
/// Formula mirrors isoProject() in rhai-live-core.js:
///   screen_x = origin_x + (pt.x - pt.z) * scale * 0.866
///   screen_y = origin_y + (pt.x + pt.z) * scale * 0.5 - pt.y * scale
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IsoProjected {
    pub x: f32,
    pub y: f32,
}

pub fn iso_project(pt: Vec3, scale: f32, origin_x: f32, origin_y: f32) -> IsoProjected {
    IsoProjected {
        x: origin_x + (pt.x - pt.z) * scale * 0.866,
        y: origin_y + (pt.x + pt.z) * scale * 0.5 - pt.y * scale,
    }
}

/// Verification depth layer — the z-axis semantic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ZLayer {
    Document   = 0,
    Pipeline   = 1,
    Constraint = 2,
    Legal      = 3,
    FormalProof = 4,
    Attestation = 5,
}

impl ZLayer {
    pub fn base_z(&self) -> f32 {
        (*self as u8 as f32) * 136.0   // lane_gap per layer
    }

    pub fn color(&self) -> &'static str {
        match self {
            ZLayer::Document    => "#334155",
            ZLayer::Pipeline    => "#1d4ed8",
            ZLayer::Constraint  => "#7c3aed",
            ZLayer::Legal       => "#b91c1c",
            ZLayer::FormalProof => "#0f766e",
            ZLayer::Attestation => "#b45309",
        }
    }
}

/// Visual specification for a domain type.
pub struct VisualizationSpec {
    /// Maps to SEMANTIC_CATEGORIES key in rhai-live-core.js.
    pub semantic_type: &'static str,
    /// Verification depth layer.
    pub z_layer: ZLayer,
    /// Rhai DSL snippet that renders this object in the isometric view.
    /// Must parse to a non-empty graph (enforced by lint test).
    pub rhai_dsl: &'static str,
    /// Human-readable description for docgen.
    pub description: &'static str,
}

/// All PRD-6 and PRD-7 domain types implement this trait.
pub trait HasVisualization {
    fn viz_spec() -> VisualizationSpec;
}
```

### 4.2 Animation Transform Record

A typed record of a node movement (maps to `<animateTransform>` in the SVG output):

```rust
/// A single frame in an isometric animation sequence.
pub struct IsoTransform {
    pub node_id: String,
    pub from: Vec3,
    pub to: Vec3,
    pub duration_ms: u32,
    pub easing: IsoEasing,
}

pub enum IsoEasing {
    Linear,
    EaseOut,
    Spring,  // for confidence-driven transitions
}
```

The Rust `PipelineGraph::update()` will produce `Vec<IsoTransform>` on each state change, which the HTML renderer serializes into `<animateTransform>` elements matching the JS `animationMs: 460` convention.

### 4.3 manim-Style Path Recording

For long-form animation sequences (data processing demos, documentation videos):

```rust
/// A sequence of transforms forming one logical animation.
pub struct IsoAnimationPath {
    pub title: String,
    pub frames: Vec<IsoFrame>,
}

pub struct IsoFrame {
    pub t_ms: u32,            // absolute time from start
    pub transforms: Vec<IsoTransform>,
    pub confidence_overlay: Option<f32>,   // confidence arc overlay
    pub annotation: Option<String>,        // frame caption
}

impl IsoAnimationPath {
    /// Serialize to a manim-compatible Python script stub.
    /// Outputs Scene subclass with one Indicate/Transform per frame.
    pub fn to_manim_script(&self) -> String { ... }

    /// Serialize to SVG SMIL animation sequence.
    pub fn to_smil_svg(&self) -> String { ... }
}
```

---

## 5. Lint Test Contract

`crates/ledger-core/tests/iso_lint.rs` — one test per PRD-6/7 object:

```rust
// Pattern for every object:
#[test]
fn <type_name>_viz_spec_parses() {
    let spec = <TypeViz>::viz_spec();
    // 1. DSL parses to non-empty graph
    let graph = mdbook_rhai_mermaid::parser::parse(spec.rhai_dsl);
    assert!(!graph.order.is_empty(), "{} DSL produced empty graph", stringify!(<TypeViz>));
    // 2. z_layer is in valid range
    assert!(spec.z_layer as u8 <= 5);
    // 3. semantic_type maps to a known category
    assert!(KNOWN_SEMANTIC_TYPES.contains(&spec.semantic_type));
    // 4. description is non-empty
    assert!(!spec.description.is_empty());
}
```

21 tests total (15 PRD-6 objects + 6 PRD-7 objects).

---

## 6. Rhai DSL Canonical Snippets

Each snippet is a `pub const` in `crates/ledger-core/src/iso_objects.rs`. The full set is recorded in `book/src/iso-pipeline-objects.md` and in `_docs/iso-pipeline.tomllmd`.

Example — `PIPELINE_FULL_DSL` (the complete PRD-6 + PRD-7 flow in one scene):

```rhai
fn document_received() -> check_constraints
fn check_constraints() -> validate
if confidence > 0.0 -> verify_legal
match legal_result => Z3Result::Satisfied -> classify
match legal_result => Z3Result::Violated -> needs_review
match legal_result => Z3Result::Unknown -> classify
fn classify() -> reconcile
fn reconcile() -> evaluate_commit_gate
match commit_gate => CommitGate::Approved -> commit
match commit_gate => CommitGate::PendingOperator -> operator_review
match commit_gate => CommitGate::Blocked -> needs_review
fn commit() -> invariant_ledger
fn invariant_ledger() -> workbook_export
```

---

## 7. Docgen and `.tomllmd` Dynamic Section

### 7.1 mdBook Chapter (`book/src/iso-pipeline-objects.md`)

One section per PRD layer. Each section contains:
- The object's `VisualizationSpec` fields rendered as a table
- The Rhai DSL snippet rendered as a live diagram block (processed by `mdbook-rhai-mermaid`)
- The z-layer color band shown as a CSS swatch

### 7.2 `.tomllmd` Section (`_docs/iso-pipeline.tomllmd`)

Dynamic section with three summary levels and command interpolation:

```toml
[meta]
title = "Isometric Pipeline Object Visualization"
invariant = "iso-pipeline-objects"
entanglement = ["prd-6.concept", "prd-7.synergy", "prd-6-future.attest"]

[[section]]
id = "layer-model"
verbatim = """
[full layer model and z-axis semantics]
"""
executive = "Six z-layers map pipeline progress (x), confidence lift (y), and verification depth (z). Each domain type has a VisualizationSpec with semantic_type, z_layer, and a parseable Rhai DSL snippet."
epigram = "x=progress, y=confidence, z=verification depth; 21 typed objects, 21 lint tests."

[[section]]
id = "object-inventory"
verbatim = "{{ cmd: cargo test -p ledger-core --test iso_lint 2>&1 | tail -30 }}"
executive = "21 PRD-6/7 objects each have a VisualizationSpec. iso_lint.rs asserts DSL parses, z_layer ≤ 5, semantic_type is known, description is non-empty."
epigram = "cargo test iso_lint = 21 green."

[[section]]
id = "animation-model"
verbatim = "IsoTransform { from: Vec3, to: Vec3, duration_ms } → <animateTransform>. IsoAnimationPath → manim Python stub or SVG SMIL."
executive = "State transitions produce Vec<IsoTransform>. to_manim_script() outputs a Scene subclass. to_smil_svg() outputs inline SVG animation."
epigram = "PipelineState transitions → typed animation frames → manim or SMIL."
```

---

## 8. manim / Animation Backend Options

| Option | Tradeoffs |
|---|---|
| `manim` Python (generated stub) | Best output quality for video; Rust generates the `.py` Scene file; no Rust animation dep |
| SVG SMIL (`<animateTransform>`) | Already used in `rhai-live-core.js`; Rust emits the same attribute format; works in mdBook |
| `bevy` + `bevy_prototype_lyon` | Full 3D interactive; heavy dep; viable for the standalone Tauri host |
| `rerun.io` Rust SDK | Real-time 3D recording + playback; zero-copy; designed for data pipelines; best fit for "animating transformations" in a logically constrained path |
| CSS keyframe generation | Simplest; already in `animation_styles()`; extend with per-node `animation-delay` |

**Recommended path:**
- Phase 0: SVG SMIL from Rust (mirrors JS, zero new deps)
- Phase 1: `rerun.io` SDK integration for the interactive 3D path (`cargo add rerun`)
- Phase 2: manim Python stub generation for documentation video export

---

## 9. Acceptance Criteria

1. `cargo test -p ledger-core --test iso_lint` — 21 tests green, 0 failed.
2. `iso_project(Vec3 { x: 192.0, y: 0.0, z: 0.0 }, 1.0, 0.0, 0.0)` returns `IsoProjected { x: 166.27, y: 96.0 }` (mirrors JS formula).
3. `PIPELINE_FULL_DSL` parsed by `mdbook_rhai_mermaid::parser::parse()` produces a graph with ≥ 13 nodes.
4. `book/src/iso-pipeline-objects.md` is rendered by `just docgen` without error.
5. `_docs/iso-pipeline.tomllmd` section `object-inventory` interpolates `{{ cmd: cargo test ... }}` to the actual test output.
6. `IsoAnimationPath::to_smil_svg()` for a two-frame `Ingested → Validated` transition produces SVG containing `<animateTransform`.
7. `ZLayer::Legal.base_z()` returns `408.0` (3 * 136.0).
