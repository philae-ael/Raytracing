use std::ops::Range;

use crate::{
    material::{texture::Uv, MaterialId},
    math::utils::sphere_uv_from_direction,
    math::vec::Vec3,
    ray::Ray,
};

pub struct HitRecord {
    pub hit_point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub material: MaterialId,
    pub uv: Uv,
}

pub enum Hit {
    Hit(HitRecord),
    NoHit,
}

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, range: Range<f32>) -> Hit;
}

pub struct Sphere {
    pub label: Option<String>,
    pub center: Vec3,
    pub radius: f32,
    pub material: MaterialId,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, range: Range<f32>) -> Hit {
        let a = ray.direction.length_squared();
        let b_half = (ray.origin - self.center).dot(ray.direction);
        let c = (ray.origin - self.center).length_squared() - self.radius * self.radius;

        let discriminant_quarter = b_half * b_half - a * c;
        if discriminant_quarter < 0.0 {
            Hit::NoHit
        } else {
            // Either find first hit if hit is in range else, find the second hit
            let t = {
                let t = (-b_half - f32::sqrt(discriminant_quarter)) / a;
                if range.contains(&t) {
                    t
                } else {
                    let t = (-b_half + f32::sqrt(discriminant_quarter)) / a;
                    if !range.contains(&t) {
                        return Hit::NoHit;
                    }
                    t
                }
            };
            let hit_point = ray.at(t);
            let normal = (hit_point - self.center).normalize();
            let uv = sphere_uv_from_direction(normal);
            let record = HitRecord {
                hit_point,
                normal,
                t,
                material: self.material,
                uv,
            };
            Hit::Hit(record)
        }
    }
}

pub struct HittableList(pub Vec<Box<dyn Hittable>>);

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, range: std::ops::Range<f32>) -> Hit {
        let start = range.start;
        let mut end = range.end;
        let mut res = Hit::NoHit;

        for hittable in self.0.iter() {
            let range = start..end;
            if range.is_empty() {
                return Hit::NoHit;
            }

            if let Hit::Hit(record) = hittable.hit(ray, range) {
                end = record.t;
                res = Hit::Hit(record);
            }
        }
        res
    }
}
