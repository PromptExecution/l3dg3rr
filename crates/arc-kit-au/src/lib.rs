//! # arc-kit-au — Evidence Traceability Layer
//!
//! Forked from the arc-kit governance model, this crate provides structured
//! evidence tracking for bookkeeping: source documents → extracted rows →
//! transactions → classifications → model proposals → operator approvals →
//! workbook exports.
//!
//! ## Core Model
//!
//! Every node in the evidence graph has a deterministic Blake3 identity.
//! Edges represent provenance relationships. The graph can be queried for
//! complete transaction chains or missing provenance gaps.
//!
//! ## Usage
//!
//! ```rust
//! use arc_kit_au::{EvidenceGraph, EvidenceStore};
//!
//! let graph = EvidenceGraph::new();
//! assert!(graph.is_empty());
//!
//! // The graph can be persisted to disk and reloaded
//! let json = graph.to_json().unwrap();
//! let restored = EvidenceGraph::from_json(&json).unwrap();
//! assert!(restored.is_empty());
//! ```

pub mod node;
pub mod edge;
pub mod graph;
pub mod builder;
pub mod trace;
pub mod missing;
pub mod badge;
pub mod store;

pub use node::{Confidence, EvidenceNode, NodeId, NodeType};
pub use edge::{EvidenceEdge, EdgeType};
pub use graph::{EvidenceGraph, WorkQueueSummary};
pub use builder::EvidenceBuilder;
pub use trace::{EvidenceChain, EvidenceTracer};
pub use missing::{MissingElement, ProvenanceGap, ProvenanceScanner};
pub use badge::ProvenanceBadge;
pub use store::EvidenceStore;
