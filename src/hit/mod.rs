use std::ops::Range;

use crate::{
    math::vec::{Normal, Point},
    ray::Ray,
};

pub struct BaseHitRecord {
    pub hit_point: Point,
    pub normal: Normal,
    pub t: f64,
}

pub enum Hit<T = BaseHitRecord> {
    Hit(T),
    NoHit,
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, range: Range<f64>) -> Hit;
}

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, range: Range<f64>) -> Hit {
        let a = ray.direction.length_squared();
        let b_half = (ray.origin - self.center).dot(&ray.direction);
        let c = (ray.origin - self.center).length_squared() - self.radius * self.radius;

        let discriminant_quarter = b_half * b_half - a * c;
        if discriminant_quarter < 0.0 {
            Hit::NoHit
        } else {
            // Either find first hit if hit is in range else, find the second hit
            let t = {
                let t = (-b_half - f64::sqrt(discriminant_quarter)) / a;
                if range.contains(&t) {
                    t
                } else {
                    let t = (-b_half + f64::sqrt(discriminant_quarter)) / a;
                    if !range.contains(&t) {
                        return Hit::NoHit;
                    }
                    t
                }
            };
            let hit_point = ray.at(t);
            let normal = (hit_point - self.center).normalize();
            let record = BaseHitRecord {
                hit_point,
                normal,
                t,
            };
            Hit::Hit(record)
        }
    }
}
