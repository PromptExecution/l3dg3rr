//! Slint UI integration for isometric graph visualization.
//!
//! This module provides the texture bridge and UI components for rendering
//! the animated pipeline graph in a Slint window on Windows.

use crate::graph::NodeData;
use crate::layout::ForceLayout;
use crate::render::GraphRenderer;
use std::sync::{Arc, RwLock};

pub struct SlintGraphView {
    pub renderer: GraphRenderer,
    pub layout: Arc<RwLock<ForceLayout>>,
}

impl SlintGraphView {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            renderer: GraphRenderer::new(width, height),
            layout: Arc::new(RwLock::new(ForceLayout::for_pipeline())),
        }
    }

    pub fn tick(&self) {
        if let Ok(mut layout) = self.layout.write() {
            layout.tick();
        }
    }

    pub fn screen_position(&self, node_idx: usize) -> Option<(f32, f32)> {
        let layout = self.layout.read().ok()?;
        let pos = layout.position(node_idx)?;
        let screen = self.renderer.screen_position(pos.x, pos.y, pos.z);
        Some((screen.x, screen.y))
    }
}