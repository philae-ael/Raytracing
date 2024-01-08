pub mod shapelist;

use crate::{
    hit::{Hit, Hittable},
    ray::Ray,
    surface::HitPoint,
};

pub trait Aggregate {
    fn first_hit(&self, ray: Ray) -> Hit;
    fn first_hitpoint(&self, ray: Ray) -> HitPoint;
}

impl<T> Hittable for T
where
    T: Aggregate,
{
    fn hit(&self, ray: Ray) -> Hit {
        self.first_hit(ray)
    }
}
