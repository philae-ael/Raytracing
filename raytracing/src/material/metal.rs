use glam::Vec3;
use rand::prelude::Distribution;

use crate::{
    math::{
        distributions::{UnitBall3, UnitBall3PolarMethod},
        vec::{RefrReflVecExt, Vec3AsNonZero, RgbAsVec3Ext},
    },
    ray::Ray,
    shape::local_info,
};

use super::{texture::Texture, Material, Scattered};

pub struct Metal {
    pub roughness: f32,
    pub texture: Box<dyn Texture>,
}

impl Material for Metal {
    fn scatter(
        &self,
        ray: Ray,
        record: &local_info::Full,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Scattered {
        let ray_direction = ray.direction.reflect(record.normal);
        let fuziness = self.roughness
            * Vec3::from_array(UnitBall3::<UnitBall3PolarMethod>::default().sample(rng));
        let ray_direction = (ray_direction + fuziness)
            .as_non_zero(0.01)
            .unwrap_or(ray_direction);

        let ray_out = if ray_direction.dot(record.normal) > 0.0 {
            Some(Ray::new(record.pos, ray_direction))
        } else {
            None
        };

        Scattered {
            ray_out,
            albedo: self.texture.color(record.uv),
        }
    }

    fn reflection(&self) -> Option<Vec3> {
        Some(self.texture.color([0., 0.]).vec())
    }

    fn diffuse(&self) -> Option<Vec3> {
        None
    }
}
