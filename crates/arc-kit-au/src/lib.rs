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

pub mod badge;
pub mod builder;
pub mod edge;
pub mod graph;
pub mod missing;
pub mod node;
pub mod store;
pub mod trace;

pub use badge::ProvenanceBadge;
pub use builder::EvidenceBuilder;
pub use edge::{EdgeType, EvidenceEdge};
pub use graph::{EvidenceGraph, WorkQueueSummary};
pub use missing::{MissingElement, ProvenanceGap, ProvenanceScanner};
pub use node::{Confidence, EvidenceNode, NodeId, NodeType};
pub use store::EvidenceStore;
pub use trace::{EvidenceChain, EvidenceTracer};
