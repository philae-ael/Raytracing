use crate::{
    hit::{Hit, Hittable},
    ray::Ray,
};

use super::Aggregate;

pub struct ShapeList(pub Vec<Box<dyn Hittable + Sync>>);

impl Aggregate for ShapeList {
    fn first_hit(&self, mut ray: Ray) -> Hit {
        let mut res = Hit::NoHit;

        for hittable in self.0.iter() {
            if ray.range().is_empty() {
                return Hit::NoHit;
            }

            if let Hit::Hit(record) = hittable.hit(ray) {
                ray.bounds.1 = record.t;
                res = Hit::Hit(record);
            }
        }
        res
    }

    fn first_hitpoint(&self, _ray: crate::ray::Ray) -> crate::surface::HitPoint {
        todo!()
    }
}
