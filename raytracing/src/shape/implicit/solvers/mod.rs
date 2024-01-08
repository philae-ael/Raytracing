//! The solvers used to solve intersection on implicit surfaces are here.
mod newton;

use crate::ray::Ray;
use glam::Vec3;

/// data about the intersection of a ray an an implicit surface
pub struct ImplicitSolution {
    /// Time of intersection
    pub t: f32,
    /// Position of intersection
    pub hit_point: Vec3,
    /// normal of implicit surface at intersection
    ///
    /// This will be wrong or explode if the implicit function is not smooth
    pub normal: Vec3,
}

/// An algorithm to solve the intersection problem of a ray an an implicit surface
///
/// Note that this is actually quite easy because we only need to find a `t` such that
/// `f(ray.at(t))` is near 0 (it's a 1D optimization problem)
pub trait ImplicitSolver {
    /// This solve the problem
    ///
    /// Return None if anything goes wrong
    fn solve<F: Fn(Vec3) -> f32>(&self, f: F, ray: Ray) -> Option<ImplicitSolution>;
}

pub use newton::NewtonSolver;
