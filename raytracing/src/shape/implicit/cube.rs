

use crate::math::point::Point;

use super::ImplicitSurface;

/// An implicit cube
///
/// It should not work well on edges because it's not smooth there
pub struct Cube {
    /// The size of an edge of the cube
    pub size: f32,
    /// The center of the cube
    pub origin: Point,
}

impl ImplicitSurface for Cube {
    fn impl_f(&self, p: Point) -> f32 {
        (p - self.origin).abs().max_element() - self.size / 2.0
    }
}
