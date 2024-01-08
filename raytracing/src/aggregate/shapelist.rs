use crate::{
    ray::Ray,
    shape::{local_info, FullIntersectionResult, IntersectionResult, MinIntersectionResult, Shape},
};

use super::Aggregate;

#[derive(Default)]
pub struct ShapeList(pub Vec<Box<dyn Shape + Sync>>);
impl Aggregate for ShapeList {}

impl Shape for ShapeList {
    fn intersection_full(&self, mut ray: Ray) -> FullIntersectionResult {
        let mut res = IntersectionResult::NoIntersection;

        for hittable in self.0.iter() {
            if ray.range().is_empty() {
                return IntersectionResult::NoIntersection;
            }

            if let IntersectionResult::Instersection(record) = hittable.intersection_full(ray) {
                ray.bounds.1 = record.t;
                res = IntersectionResult::Instersection(record);
            }
        }
        res
    }

    fn intersect_bare(&self, _ray: Ray) -> MinIntersectionResult {
        todo!()
    }

    fn local_information(&self, _p: glam::Vec3) -> Option<local_info::Full> {
        todo!()
    }
}
