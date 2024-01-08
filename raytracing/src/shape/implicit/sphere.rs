use crate::math::point::Point;

use super::ImplicitSurface;

/// An implicit sphere
pub struct Sphere {
    pub radius: f32,
    pub origin: Point,
}

impl ImplicitSurface for Sphere {
    fn impl_f(&self, p: Point) -> f32 {
        (self.origin - p).length() - self.radius * self.radius
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::{
        material::MaterialId,
        math::point::Point,
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
            origin: Point::ORIGIN,
        };

        assert!(sphere.impl_f(Point::new(2.0, 0.0, 0.0)).abs() > eps);
        assert!(sphere.impl_f(Point::new(0.0, 1.0, 0.0)).abs() < eps);
    }

    #[test]
    fn sphere_hit() {
        let sphere = ImplicitShape {
            surface: Sphere {
                radius: 0.5,
                origin: Point::new(1.0, 0.0, 0.0),
            },
            solver: NewtonSolver {
                eps: 0.00001,
                max_iter: 10,
            },
            material: MaterialId(0),
        };

        let hit = sphere.intersection_full(Ray::new(Point::ORIGIN, Vec3::X));
        match hit {
            IntersectionResult::Instersection(h) => {
                assert!(
                    h.local_info
                        .pos
                        .vec()
                        .distance_squared(Vec3::new(0.5, 0., 0.))
                        < sphere.solver.eps
                );
            }
            IntersectionResult::NoIntersection => panic!("{hit:?}"),
        }
    }
}
