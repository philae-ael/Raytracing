//! This submodule contains everything about implicit surfaces
//!

mod anonymous;
mod cube;
pub mod solvers;
mod sphere;

use self::solvers::{ImplicitSolution, ImplicitSolver};
use super::{
    local_info, shape::RayIntersection, FullIntersectionResult, IntersectionResult,
    MinIntersectionResult, Shape,
};
use crate::{material::MaterialId, ray::Ray};
use glam::Vec3;

/// Defines a surface by an implicit parametrisation, given by impl_f
pub trait ImplicitSurface {
    fn impl_f(&self, p: Vec3) -> f32;
}

/// Contains all the information needed to make an implicit surface a shape
pub struct ImplicitShape<Surf: ImplicitSurface, Solv: ImplicitSolver> {
    pub surface: Surf,
    pub solver: Solv,
    /// The material of the surface. This material cannot depend on UV coordinates
    pub material: MaterialId,
}

impl<Surf: ImplicitSurface, Solv: ImplicitSolver> Shape for ImplicitShape<Surf, Solv> {
    fn intersection_full(&self, ray: Ray) -> FullIntersectionResult {
        if let Some(ImplicitSolution {
            t,
            hit_point,
            normal,
        }) = self.solver.solve(|x| self.surface.impl_f(x), ray)
        {
            IntersectionResult::Instersection(RayIntersection {
                t,
                local_info: local_info::Full {
                    pos: hit_point,
                    normal,
                    material: self.material,
                    uv: [0.0, 0.0],
                },
            })
        } else {
            IntersectionResult::NoIntersection
        }
    }

    /// Note: this isn't faster than intersection_full because i don't want to code each solver for each local_info variant
    fn intersect_bare(&self, ray: Ray) -> MinIntersectionResult {
        if let IntersectionResult::Instersection(RayIntersection {
            t,
            local_info: local_info::Full { pos, .. },
        }) = self.intersection_full(ray)
        {
            IntersectionResult::Instersection(RayIntersection {
                t,
                local_info: local_info::Minimum { pos },
            })
        } else {
            IntersectionResult::NoIntersection
        }
    }

    fn local_information(&self, _p: Vec3) -> Option<local_info::Full> {
        todo!()
    }
}

pub use anonymous::Anonymous;
pub use cube::Cube;
pub use sphere::Sphere;
