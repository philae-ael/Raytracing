use glam::Vec3;
use image::Rgb;

use crate::{
    math::vec::{RefrReflVecExt, RgbAsVec3Ext, Vec3AsRgbExt},
    ray::Ray,
    shape::local_info,
};

use super::{Material, Scattered};

/// Based of the [http://artis.imag.fr/~Cyril.Soler/DEA/NonPhotoRealisticRendering/Papers/p447-gooch.pdf](original paper on Gooch shading)
pub struct Gooch {
    pub diffuse: Rgb<f32>,
    pub smoothness: f32,
    pub light_dir: Vec3,
    pub yellow: Rgb<f32>,
    pub blue: Rgb<f32>,
}

impl Material for Gooch {
    fn scatter(
        &self,
        ray: Ray,
        record: &local_info::Full,
        _rng: &mut rand::rngs::ThreadRng,
    ) -> Scattered {
        let light_dir = self.light_dir.normalize();
        let gooch_factor = (1. + record.normal.dot(light_dir)) / 2.;
        let alpha = 0.4;
        let beta = 0.6;
        let cool = alpha * self.blue.vec();
        let warm = beta * self.yellow.vec();
        let diffuse = gooch_factor * cool + (1.0 - gooch_factor) * warm;

        let omega = light_dir
            .reflect(record.normal)
            .dot(-ray.direction)
            .clamp(0.0, 1.0);
        let specular = omega.powf(self.smoothness);

        let color = specular * Vec3::ONE + diffuse;
        Scattered {
            albedo: color.rgb(),
            ray_out: None,
        }
    }
}
