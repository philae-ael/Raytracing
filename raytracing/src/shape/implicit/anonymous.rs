//! An implicit shape from an anonymous function



use crate::math::point::Point;

use super::ImplicitSurface;

/// A thin wrapper around a function that allow to draw arbitrary implicit surface
pub struct Anonymous<F: Fn(Point) -> f32>(pub F);

impl<F: Fn(Point) -> f32> ImplicitSurface for Anonymous<F> {
    fn impl_f(&self, p: Point) -> f32 {
        self.0(p)
    }
}
