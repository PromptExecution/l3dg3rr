use crate::graph::{create_pipeline_edges, create_pipeline_nodes};
use glam::{Vec3, Vec2};
use std::collections::HashMap;

pub fn iso_project(p: Vec3, scale: f32, origin: Vec2) -> Vec2 {
    Vec2 {
        x: origin.x + (p.x - p.z) * scale * 0.866,
        y: origin.y + (p.x + p.z) * scale * 0.5 - p.y * scale,
    }
}

pub struct ForceLayout {
    positions: HashMap<usize, Vec3>,
    #[allow(dead_code)]
    velocities: HashMap<usize, Vec3>,
}

impl ForceLayout {
    pub fn new() -> Self {
        let positions = HashMap::new();
        let velocities = HashMap::new();
        Self {
            positions,
            velocities,
        }
    }

    pub fn for_pipeline() -> Self {
        let nodes = create_pipeline_nodes();
        let _edges = create_pipeline_edges();
        let mut positions = HashMap::new();
        let velocities = HashMap::new();
        for (i, _node) in nodes.iter().enumerate() {
            let angle = (i as f32) * std::f32::consts::TAU / nodes.len() as f32;
            positions.insert(
                i,
                Vec3::new(angle.cos() * 100.0, 0.0, angle.sin() * 100.0),
            );
        }
        Self {
            positions,
            velocities,
        }
    }

    pub fn tick(&mut self) {
        let nodes: Vec<_> = self.positions.keys().copied().collect();
        for i in nodes {
            if let Some(pos) = self.positions.get_mut(&i) {
                let center = Vec3::ZERO;
                let to_center = center - *pos;
                let dist = to_center.length();
                if dist > 1.0 {
                    let force = to_center.normalize() * dist * 0.01;
                    *pos += force;
                }
            }
        }
    }

    pub fn position(&self, node_idx: usize) -> Option<Vec3> {
        self.positions.get(&node_idx).copied()
    }

    pub fn all_positions(&self) -> &HashMap<usize, Vec3> {
        &self.positions
    }
}

impl Default for ForceLayout {
    fn default() -> Self {
        Self::new()
    }
}