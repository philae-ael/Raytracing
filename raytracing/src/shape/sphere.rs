use glam::Vec3;

use crate::{
    material::MaterialId,
    math::{bounds::Bounds, distributions::sphere_uv_from_direction, point::Point},
    ray::Ray,
};

use super::{
    local_info,
    shape::{MinIntersectionResult, RayIntersection},
    FullIntersectionResult, IntersectionResult, Shape,
};

/// A simple sphere shape.
///
/// Normals are pointing outwards if `radius` is positive, and are reversed if `radius` is negative
#[derive(Debug)]
pub struct Sphere {
    pub center: Point,
    pub radius: f32,
    pub material: MaterialId,
}

impl Shape for Sphere {
    fn intersection_full(&self, ray: Ray) -> FullIntersectionResult {
        if let IntersectionResult::Intersection(RayIntersection {
            t,
            local_info: local_info::Minimum { pos },
        }) = self.intersect_bare(ray)
        {
            let normal = self.radius.signum() * (pos - self.center).normalize();
            let uv = sphere_uv_from_direction(normal);
            IntersectionResult::Intersection(RayIntersection {
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
        IntersectionResult::Intersection(RayIntersection {
            t,
            local_info: local_info::Minimum { pos },
        })
    }

    fn bounding_box(&self) -> Bounds {
        let v = self.radius * Vec3::ONE;
        Bounds::from_points(&[self.center - v, self.center + v])
    }
}
