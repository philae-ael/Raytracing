use glam::Vec3;

use crate::{material::MaterialId, math::distributions::sphere_uv_from_direction, ray::Ray};

use super::{
    local_info,
    shape::{MinIntersectionResult, RayIntersection},
    FullIntersectionResult, IntersectionResult, Shape,
};

/// A simple sphere shape.
///
/// Normals are pointing outwards if `radius` is positive, and are reversed if `radius` is negative
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: MaterialId,
}

impl Shape for Sphere {
    fn intersection_full(&self, ray: Ray) -> FullIntersectionResult {
        if let IntersectionResult::Instersection(RayIntersection {
            t,
            local_info: local_info::Minimum { pos },
        }) = self.intersect_bare(ray)
        {
            let normal = self.radius.signum() * (pos - self.center).normalize();
            let uv = sphere_uv_from_direction(normal);
            IntersectionResult::Instersection(RayIntersection {
                t,
                local_info: local_info::Full {
                    pos,
                    normal,
                    material: self.material,
                    uv,
                },
            })
        } else {
            IntersectionResult::NoIntersection
        }
    }

    fn intersect_bare(&self, ray: Ray) -> MinIntersectionResult {
        let a = ray.direction.length_squared();
        let b_half = (ray.origin - self.center).dot(ray.direction);
        let c = (ray.origin - self.center).length_squared() - self.radius * self.radius;

        let discriminant_quarter = b_half * b_half - a * c;
        let t = if discriminant_quarter > 0.0 {
            // Either find first hit if hit is in range else, find the second hit
            let ta = (-b_half + f32::sqrt(discriminant_quarter)) / a;
            let tb = (-b_half - f32::sqrt(discriminant_quarter)) / a;
            let (ta, tb) = if ta > tb { (tb, ta) } else { (tb, ta) };
            let range = ray.range();
            if range.contains(&ta) {
                ta
            } else if range.contains(&tb) {
                tb
            } else {
                return IntersectionResult::NoIntersection;
            }
        } else {
            return IntersectionResult::NoIntersection;
        };

        let pos = ray.at(t);
        IntersectionResult::Instersection(RayIntersection {
            t,
            local_info: local_info::Minimum { pos },
        })
    }

    fn local_information(&self, _p: Vec3) -> Option<local_info::Full> {
        todo!()
    }
}
