use glam::Vec3;
use rand::{distributions, prelude::Distribution};

use crate::{
    math::vec::{RefrReflVecExt, RgbAsVec3Ext},
    ray::Ray,
    shape::local_info,
};

use super::{texture::Texture, Material, Scattered};

pub struct Dielectric {
    pub texture: Box<dyn Texture>,
    pub ior: f32,
}

impl Material for Dielectric {
    fn scatter(
        &self,
        ray: Ray,
        record: &local_info::Full,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Scattered {
        let sampler = distributions::Uniform::new_inclusive(0.0, 1.0);
        let cos_incident = record.normal.dot(ray.direction);
        // IOR is the ratio n1/n2 where n2 is the indice of refraction of the material and n1 the indice of refraction of the medium
        // If the view vector and the normal are opposed (and cos_incident < 0.0) it means the ray goes from outside to inside the material thus ior = n1/n2
        // Otherwise, the ray goes from inside the material to outside the material and ior is ior = n2/n1
        // Also makes sure that the normal is facing us
        let (n, normal) = if cos_incident < 0.0 {
            (1.0 / self.ior, record.normal)
        } else {
            (self.ior / 1.0, -record.normal)
        };

        // See https://en.wikipedia.org/wiki/Fresnel_equations
        // And https://en.wikipedia.org/wiki/Schlick%27s_approximation
        fn reflectance(cos: f32, ref_idx: f32) -> f32 {
            let r0 = {
                let r = (1. - ref_idx) / (1. + ref_idx);
                r * r
            };
            r0 + (1. - r0) * f32::powi(1. - cos.abs(), 5)
        }

        let reflected = ray.direction.reflect(normal);
        let dir_out = if let Some(refracted) = ray.direction.refract(normal, n) {
            // check reflectance to see whether a reflection ray of a refraction ray should be cast
            let reflectance = reflectance(cos_incident, n);
            let sample = sampler.sample(rng);

            if sample < reflectance {
                reflected
            } else {
                refracted
            }
        } else {
            reflected
        };

        let ray_out = Ray::new(record.pos, dir_out);
        Scattered {
            ray_out: Some(ray_out),
            albedo: self.texture.color(record.uv),
        }
    }

    fn transmission(&self) -> Option<(f32, Vec3)> {
        Some((self.ior, self.texture.color([0.0, 0.0]).vec()))
    }

    fn reflection(&self) -> Option<Vec3> {
        None
    }

    fn diffuse(&self) -> Option<Vec3> {
        Some(self.texture.color([0.0, 0.0]).vec())
    }
}
