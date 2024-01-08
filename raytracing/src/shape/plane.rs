use glam::Vec3;

use crate::{
    material::MaterialId,
    math::{bounds::Bounds, point::Point},
    ray::Ray,
};

use super::{
    local_info, shape::RayIntersection, FullIntersectionResult, IntersectionResult, Shape,
};

pub struct Plane {
    pub origin: Point,
    pub normal: Vec3,
    pub material: MaterialId,
}

impl Shape for Plane {
    fn intersection_full(&self, ray: Ray) -> FullIntersectionResult {
        let t = -(ray.origin - self.origin).dot(self.normal) / ray.direction.dot(self.normal);

        if !ray.range().contains(&t) {
            return IntersectionResult::NoIntersection;
        }
        let pos = ray.at(t);

        IntersectionResult::Intersection(RayIntersection {
            local_info: local_info::Full {
                pos,
                normal: self.normal,
                material: self.material,
                uv: [0.0, 0.0],
            },
            t,
        })
    }

    fn bounding_box(&self) -> Bounds {
        Bounds {
            origin: Point(f32::NEG_INFINITY * Vec3::ONE),
            end: Point(f32::INFINITY * Vec3::ONE),
        }
    }

    fn intersect_bare(&self, _ray: Ray) -> super::MinIntersectionResult {
        todo!()
    }
}
