pub mod calendar;
pub mod classify;
pub mod constraints;
pub mod document;
pub mod document_shape;
pub mod filename;
pub mod fs_meta;
pub mod graph;
pub mod ingest;
pub mod journal;
pub mod ledger_ops;
pub mod legal;
pub mod layout;
pub mod manifest;
pub mod pipeline;
pub mod render;
pub mod rule_registry;
pub mod slint_viz;
pub mod tags;
pub mod validation;
pub mod verify;
pub mod visualize;
pub mod workbook;
pub mod workflow;

pub use graph::{NodeData, EdgeData, create_pipeline_nodes, create_pipeline_edges};
pub use layout::{ForceLayout, iso_project};
pub use render::GraphRenderer;
pub use slint_viz::SlintGraphView;

#[cfg(test)]
mod integration_tests;
