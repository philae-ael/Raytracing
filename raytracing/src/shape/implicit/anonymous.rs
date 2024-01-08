//! An implicit shape from an anonymous function

use glam::Vec3;

use super::ImplicitSurface;

/// A thin wrapper around a function that allow to draw arbitrary implicit surface
pub struct Anonymous<F: Fn(Vec3) -> f32>(pub F);

impl<F: Fn(Vec3) -> f32> ImplicitSurface for Anonymous<F> {
    fn impl_f(&self, p: Vec3) -> f32 {
        self.0(p)
    }
}
