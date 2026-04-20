# Workflow

The workflow module provides a TOML-based DSL for defining pipeline stages and transitions.

## WorkflowToml

```rust
pub struct WorkflowToml {
    pub version: String,
    pub state: Vec<StateDef>,
    pub transitions: Vec<TransitionDef>,
}
```

## StateDef

```rust
pub struct StateDef {
    pub id: String,
    pub description: String,
    pub verbs: Vec<VerbDef>,
}
```

## TransitionDef

```rust
pub struct TransitionDef {
    pub from: String,
    pub to: String,
    pub event: String,
    pub guard: Option<String>,
}
```

## Compilation

The workflow DSL compiles to:
- **Mermaid**: Visual diagram generation
- **Rhai**: Runtime execution FSM
- **Rust enum**: Compile-time type safety

## Example

```toml
[workflow]
version = "1.0"

[[state]]
id = "Ingested"
description = "Document parsed"

[[state]]
id = "Validating"
description = "Checking integrity"

[[transition]]
from = "Ingested"
to = "Validating"
event = "validate"
```