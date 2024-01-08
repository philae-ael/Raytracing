use glam::Vec3;

use super::ImplicitSurface;

/// An implicit sphere
pub struct Sphere {
    pub radius: f32,
    pub origin: Vec3,
}

impl ImplicitSurface for Sphere {
    fn impl_f(&self, p: Vec3) -> f32 {
        self.origin.distance_squared(p) - self.radius * self.radius
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::{
        material::MaterialId,
        ray::Ray,
        shape::{
            implicit::{solvers::NewtonSolver, ImplicitShape, ImplicitSurface, Sphere},
            shape::{IntersectionResult, Shape},
        },
    };

    #[test]
    fn sphere_impl_surf() {
        let eps = 0.01;
        let sphere = Sphere {
            radius: 1.,
            origin: Vec3::ZERO,
        };

        assert!(
            sphere
                .impl_f(Vec3 {
                    x: 2.0,
                    y: 0.0,
                    z: 0.0
                })
                .abs()
                > eps
        );
        assert!(
            sphere
                .impl_f(Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0
                })
                .abs()
                < eps
        );
    }

    #[test]
    fn sphere_hit() {
        let sphere = ImplicitShape {
            surface: Sphere {
                radius: 0.5,
                origin: Vec3::X,
            },
            solver: NewtonSolver {
                eps: 0.00001,
                max_iter: 10,
            },
            material: MaterialId(0),
        };

        let hit = sphere.intersection_full(Ray::new(Vec3::ZERO, Vec3::X));
        match hit {
            IntersectionResult::Instersection(h) => {
                assert!(
                    h.local_info.pos.distance_squared(Vec3::new(0.5, 0., 0.)) < sphere.solver.eps
                );
            }
            IntersectionResult::NoIntersection => panic!("{hit:?}"),
        }
    }
}
