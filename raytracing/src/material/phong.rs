use glam::Vec3;

use crate::{
    math::vec::{RefrReflVecExt, RgbAsVec3Ext, Vec3AsRgbExt},
    ray::Ray,
    shape::local_info, color::Rgb,
};

use super::{Material, Scattered};

pub struct Phong {
    pub ambiant: Rgb,
    pub albedo: Rgb,
    pub smoothness: f32,
    pub light_dir: Vec3,
}

impl Material for Phong {
    fn scatter(
        &self,
        ray: Ray,
        record: &local_info::Full,
        _rng: &mut rand::rngs::ThreadRng,
    ) -> Scattered {
        let light_dir = self.light_dir.normalize();
        let diffuse = record.normal.dot(light_dir) * self.albedo.vec();
        let omega = -light_dir.reflect(record.normal).dot(ray.direction);
        let specular = omega.powf(self.smoothness);

        let color = specular * Vec3::ONE + diffuse + self.ambiant.vec();
        Scattered {
            albedo: color.rgb(),
            ray_out: None,
        }
    }
}
