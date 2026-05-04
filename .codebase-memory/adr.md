## PURPOSE

This ADR captures the design of `.tomllmd` — a compound structured document format that is the distillation product of an auto-research pipeline. It is the core memoization and knowledge transfer invariant across the l3dg3rr ↔ b00t mesh.

## ARCHITECTURE

```
autoresearch [markdown] + tomllm => .tomllmd
```

The pipeline:

1. **Autoresearch** (ralph loop or grok/ask/digest chain) produces markdown at varying levels of detail
2. **tomllmd compiler** reads markdown + session context + role/skill invariants and emits `.tomllmd`
3. **tomllmd files** are valid TOML with enriched conventions, storing results at multiple summary levels (verbatim, executive, epigram)
4. **Compounding** — two or more `.tomllmd` files can be merged by a local LLM (sm0l/ch0nky tier) into a higher-order meta-learn datum

## STACK

### .tomllmd Format (extends .tomllm)

`.tomllmd` = valid TOML with the same enriched comment conventions as `.tomllm` (🤓, 💡例, ⚠️需, 🔗参, 🚩, 🦨, 💩, 🐛, @epiphany, @tribal) plus:

**Command interpolation:** Lines containing `{{ cmd:... }}` are rendered at read time with live command output:
```toml
live_data = "{{ cmd: cargo test -p ledger-core --lib 2>&1 | tail -5 }}"
```

Rendering is like PHP — the template is evaluated in a sandboxed environment, with role/tier-based access control. A `ch0nky` agent gets full output; a `sm0l` agent gets a compressed summary; a `frontier` agent gets raw with provenance.

**Summary levels (per section):**
```toml
# b00t:map v1
# summary: l3dg3rr MCP provider architecture
# tags: mcp, provider, architecture, ledgerr
# tier: ch0nky
# complexity: 7

[compounding]
  sources = ["b00t-provider.tomllmd", "just-provider.tomllmd"]
  merge_strategy = "union_by_action_desc"
  produces = "external-mcp-invariant.tomllmd"

[sections.architecture]
  verbatim = """
  (full technical markdown)
  """
  executive = """
  McpProvider trait is the invariant. Three stdio providers exist.
  """
  epigram = """
  McpProvider: one trait to route them all.
  """

[sections.results]
  [[sections.results.tool_inventory]]
    name = "b00t_up"
    provider = "b00t"
    actions = ["up"]
    detail = { verbatim = "...", executive = "...", epigram = "..." }
```

### Entangled Invariant Typing

Every `.tomllmd` declares its `entangled_*` edges — these are type-validated references to other datums. The entanglement validation system (`entanglement.rs`) already enforces that `name.type` suffix matches the referenced datum's `DatumType`. This creates a verifiable capability graph:

```toml
[b00t]
name = "l3dg3rr-mcp-provider"
type = "mcp"
hint = "Generic MCP provider trait with b00t/just/ir0ntology implementations"
entangled_mcp = ["b00t-mcp.mcp", "just-mcp.mcp", "irontology.mcp"]
entangled_cli = ["b00t.cli", "just.cli"]
```

### Compounding (Future)

When a sm0l or ch0nky local LLM is present, two or more `.tomllmd` at different summary levels can be combined:

```toml
[compounding]
  sources = [
    "xero-oauth-flow.tomllmd",       # full verbatim
    "b00t-mcp-registration.tomllmd"  # executive only
  ]
  merge_strategy = "langchain_map_reduce"
  produces = "mcp-oauth-provider.tomllmd"
  llm_tier = "ch0nky"
  context_window_tokens = 128000
```

The LLM reads both files (at their respective summary levels), identifies the invariant patterns, and writes a new compound `.tomllmd` that captures the emergent knowledge. This is the `tomllmd` auto-compounding feedback loop: research → distill → memoize → compound → research again.

### Rendering Pipeline

```
Raw .tomllmd
  └── Command interpolation ({{ cmd: }})  →  Resolved .tomllmd
        └── Summary level selection
              ├── Agent role (executive.role.tomllm cognitive tiers)
              ├── Skill invariant tags
              ├── Entangled capability graph
              └── Session context (budget, tokens remaining)
                    └── Rendered document fed to LLM
```

## PATTERNS

### P1: Authors
- Pipeline: Autoresearch [markdown] → tomllmd → [compound] → tomllmd
- Each step preserves: source provenance, confidence, LLM model metadata

### P2: Summary Level Selection
- `verbatim` — full technical detail, used during implementation/authoring
- `executive` — compressed summary, used during routing and orchestration
- `epigram` — one-liner, used in tools/list descriptions and routing headers
- Selection is driven by: `tier` (sm0l/ch0nky/frontier), `entangled_*` type, available context window

### P3: Entanglement as Invariant
- Every cross-datum reference is a typed edge: `name.type`
- Validation fails if type doesn't match (`entanglement.rs:parse_entanglement_ref`)
- Datum graph DOT output visualizes the full invariant mesh

### P4: Command Rendered
- `{{ cmd: ... }}` interpolation at read time, not write time
- Sandboxed execution with role-based output truncation
- Fails closed on error (returns error message, not raw command output)

## TRADEOFFS

| Decision | Rationale |
|----------|-----------|
| TOML as base format | Serde-native, existing datum pipeline parses it, .tomllm already established |
| Summary levels in single file | Avoids file explosion; LLM can select level at read time based on context budget |
| Command interpolation at read time | Keeps files deterministic on disk; live state is ephemeral and session-scoped |
| Entanglement type suffix | Existing `entanglement.rs` already validates this; zero new infrastructure |
| Compounding deferred to LLM | The merge logic is inherently semantic — a heuristic approach would miss emergent patterns |
| Role tier drives summary depth | Simple, follows existing executive.role.tomllm cognitive tier routing; no new config needed |

## PHILOSOPHY

`.tomllmd` is the **memoization layer** for the l3dg3rr ↔ b00t agent mesh. It sits between the raw research output (markdown) and the running system (MCP tools, soul commands, datums). The invariant is: any agent at any tier can fetch any `.tomllmd` and get a version appropriate to its capabilities, through the same `b00t learn <topic>` interface.

The compounding pattern ensures that knowledge doesn't stay siloed in single-discipline datums — when an LLM has capacity, it merges related `.tomllmd` files into higher-order invariants. This is the agent equivalent of a type system: composed types produce new, richer types without losing the underlying provenance.
