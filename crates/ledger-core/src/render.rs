use glam::Vec2;

pub struct GraphRenderer {
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub origin: Vec2,
}

impl GraphRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            scale: 32.0,
            origin: Vec2::new(width as f32 / 2.0, height as f32 / 2.0),
        }
    }

    pub fn screen_position(&self, x: f32, y: f32, z: f32) -> Vec2 {
        crate::layout::iso_project(
            glam::Vec3::new(x, y, z),
            self.scale,
            self.origin,
        )
    }
}