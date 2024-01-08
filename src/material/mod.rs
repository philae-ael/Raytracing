use std::path::Path;

use image::{GenericImageView, Rgb, Rgb32FImage};
use rand::{distributions::Uniform, prelude::Distribution};

use crate::{
    color,
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

/// This function makes sure v and n are opposed by giving back a flipped n if needed
fn oppose(v: &Vec3, n: &Vec3) -> Vec3 {
    if v.dot(n) >= 0.0 {
        -n
    } else {
        *n
    }
}

/// This function makes sure v is not near zero, returning n if needed
fn non_zero_or(v: Vec3, n: Vec3) -> Vec3 {
    if v.near_zero() {
        n
    } else {
        v
    }
}

pub struct Diffuse {
    pub color: Rgb<f64>,
}

impl Material for Diffuse {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rng: &mut rand::rngs::ThreadRng) -> Scattered {
        let bounce_noise = Vec3(UnitSphere3::<UnitSphere3PolarMethod>::default().sample(rng));
        let bounce_normal = oppose(&ray.direction, &record.normal);
        let bounce_direction = non_zero_or(bounce_normal + bounce_noise, bounce_normal);

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

pub struct Metal {
    pub roughness: f64,
    pub color: Rgb<f64>,
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rng: &mut rand::rngs::ThreadRng) -> Scattered {
        let ray_direction = ray.direction.reflect(&record.normal);
        let fuziness =
            self.roughness * Vec3(UnitSphere3::<UnitSphere3PolarMethod>::default().sample(rng));
        let ray_direction = non_zero_or(ray_direction + fuziness, ray_direction);

        let ray_out = if ray_direction.dot(&record.normal) > 0.0 {
            Some(Ray::new(record.hit_point, ray_direction))
        } else {
            None
        };

        Scattered {
            ray_out,
            absorption: self.color,
        }
    }
}

pub struct Dielectric {
    pub color: Rgb<f64>,
    pub ior: f64,
    pub invert_normal: bool,
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rng: &mut rand::rngs::ThreadRng) -> Scattered {
        let uniform = Uniform::new(0.0, 1.0);
        fn reflectance(cos_theta: f64, ior: f64) -> f64 {
            let r0 = (1. - ior) / (1. + ior);
            let r0 = r0 * r0;
            r0 * (1. - r0) * (1. - cos_theta).powi(5)
        }

        let normal = if self.invert_normal {
            -&record.normal
        } else {
            record.normal
        };
        let ior = if ray.direction.dot(&record.normal) < 0.0 {
            1.0 / self.ior
        } else {
            self.ior
        };

        let cos_theta = -ray.direction.dot(&normal);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ior * sin_theta > 0.0;
        let ray_out = if cannot_refract {
            Ray::new(record.hit_point, ray.direction.reflect(&normal))
        } else {
            let refracted = ray.direction.refract(&normal, ior);
            Ray::new(record.hit_point, refracted)
        };

        Scattered {
            ray_out: Some(ray_out),
            absorption: self.color,
        }
    }
}

pub struct Environment {
    pub environment: image::Rgb32FImage,
}
impl Environment {
    pub fn new(img: &Path) -> Self {
        let im = image::open(img).expect("Can't find file").to_rgb32f();
        Environment { environment: im }
    }
}

impl Material for Environment {
    fn scatter(
        &self,
        ray: &Ray,
        _record: &HitRecord,
        _rng: &mut rand::rngs::ThreadRng,
    ) -> Scattered {
        let width = self.environment.width();
        let height = self.environment.height();
        let direction = ray.direction;
        // DOESNT WORKS
        let x = (width - 1) as f64
            * (0.5 + f64::atan2(direction.x(), -direction.z()) / std::f64::consts::TAU);
        let y = (height - 1) as f64 
            * (0.5 + f64::acos(direction.y()) / std::f64::consts::TAU);
        let color = self.environment.get_pixel(x as u32, y as u32);
        Scattered {
            absorption: Rgb(color.0.map(|x| x as f64)),
            ray_out: None,
        }
    }
}
