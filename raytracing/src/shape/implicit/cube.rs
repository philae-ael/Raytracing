use glam::Vec3;

use super::ImplicitSurface;

/// An implicit cube
///
/// It should not work well on edges because it's not smooth there
pub struct Cube {
    /// The size of an edge of the cube
    pub size: f32,
    /// The center of the cube
    pub origin: Vec3,
}

impl ImplicitSurface for Cube {
    fn impl_f(&self, p: Vec3) -> f32 {
        (p - self.origin).abs().max_element() - self.size / 2.0
    }
}
