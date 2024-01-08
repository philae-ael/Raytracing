use image::Rgb;
use rand::prelude::Distribution;

use crate::{
    hit::HitRecord,
    math::{utils::*, vec::Vec3},
    ray::Ray,
};

pub struct MaterialDescriptor {
    pub label: Option<String>,
    pub material: Box<dyn Material>,
}

#[derive(Clone, Copy)]
pub struct MaterialId(pub usize);

pub struct Scattered {
    pub absorption: Rgb<f64>,
    pub ray_out: Option<Ray>,
}

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rng: &mut rand::rngs::ThreadRng) -> Scattered;
}

pub struct Diffuse {
    pub color: Rgb<f64>,
}

impl Material for Diffuse {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rng: &mut rand::rngs::ThreadRng) -> Scattered {
        // compute whether the ray in inside or outside the geometry to the ray accordingly
        let bounce_normal = if ray.direction.dot(&record.normal) >= 0.0 {
            -record.normal
        } else {
            record.normal
        };

        let bounce_noise = Vec3(UnitSphere3::<UnitSphere3PolarMethod>::default().sample(rng));
        let bounce_direction = bounce_normal + bounce_noise;
        Scattered {
            ray_out: Some(Ray {
                origin: record.hit_point,
                direction: bounce_direction,
            }),
            absorption: self.color,
        }
    }
}

pub struct Emit {
    pub color: Rgb<f64>,
}

impl Material for Emit {
    fn scatter(
        &self,
        _ray: &Ray,
        _record: &HitRecord,
        _rng: &mut rand::rngs::ThreadRng,
    ) -> Scattered {
        Scattered {
            ray_out: None,
            absorption: self.color,
        }
    }
}
