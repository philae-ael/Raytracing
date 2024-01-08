use glam::Vec3;
use rand::distributions::{self, Distribution};

use crate::{
    math::vec::{RefrReflVecExt, RgbAsVec3Ext},
    ray::Ray,
    shape::local_info,
};

use super::{texture::Texture, Material, Scattered};

pub struct Dielectric {
    pub texture: Box<dyn Texture>,
    pub ior: f32,
    pub invert_normal: bool,
}

impl Material for Dielectric {
    fn scatter(
        &self,
        ray: Ray,
        record: &local_info::Full,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Scattered {
        fn reflectance(cos: f32, ref_idx: f32) -> f32 {
            let r0 = (1. - ref_idx) / (1. + ref_idx);
            let r0 = r0 * r0;
            r0 + (1. - r0) * f32::powi(1. - cos, 5)
        }

        let uniform = distributions::Uniform::new(0.0, 1.0);
        let normal = if self.invert_normal {
            -record.normal
        } else {
            record.normal
        };

        let cos = ray.direction.dot(normal);

        let refracted = ray.direction.refract(normal, self.ior).and_then(|x| {
            if reflectance(cos, self.ior) > uniform.sample(rng) {
                None
            } else {
                Some(x)
            }
        });

        let ray_out = if let Some(refracted) = refracted {
            Ray::new(record.pos, refracted)
        } else {
            Ray::new(record.pos, ray.direction.reflect(normal))
        };
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
