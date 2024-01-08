use glam::Vec3;
use rand::prelude::Distribution;

use crate::{
    math::{
        distributions::{UnitBall3, UnitBall3PolarMethod},
        vec::{RgbAsVec3Ext, Vec3AsNonZero, Vec3SameDirExt},
    },
    ray::Ray,
    shape::local_info,
};

use super::{texture::Texture, Material, Scattered};

pub struct Diffuse {
    pub texture: Box<dyn Texture>,
}

impl Material for Diffuse {
    fn scatter(
        &self,
        ray: Ray,
        record: &local_info::Full,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Scattered {
        let bounce_noise =
            Vec3::from_array(UnitBall3::<UnitBall3PolarMethod>::default().sample(rng));
        let bounce_normal = -record.normal.same_direction(ray.direction);
        let bounce_direction = (bounce_normal + bounce_noise)
            .into_non_zero(0.01)
            .unwrap_or(bounce_normal);

        Scattered {
            ray_out: Some(Ray::new(record.pos, bounce_direction)),
            albedo: self.texture.color(record.uv),
        }
    }

    fn diffuse(&self) -> Option<Vec3> {
        Some(self.texture.color([0.0, 0.0]).vec())
    }
}
