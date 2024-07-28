use glam::Vec3;
use rand::distributions::{self, Distribution};

use crate::{ray::Ray, shape::local_info};

use super::{Material, Scattered};

pub struct MixMaterial<Mat1: Material, Mat2: Material> {
    pub p: f32,
    pub mat1: Mat1,
    pub mat2: Mat2,
}

impl<Mat1: Material, Mat2: Material> Material for MixMaterial<Mat1, Mat2> {
    fn scatter(
        &self,
        ray: Ray,
        record: &local_info::Full,
        rng: &mut rand::rngs::StdRng,
    ) -> Scattered {
        let dist = distributions::Uniform::new_inclusive(0.0, 1.0);
        let sample = dist.sample(rng);

        let p = self.p;
        if sample < p {
            self.mat1.scatter(ray, record, rng)
        } else {
            self.mat2.scatter(ray, record, rng)
        }
    }

    fn transmission(&self) -> Option<(f32, Vec3)> {
        let transm1 = self.mat1.transmission();
        let transm2 = self.mat2.transmission();
        match transm1 {
            Some((ior1, transm_color1)) => match transm2 {
                Some((ior2, transm_color2)) => Some((
                    self.p * ior1 + (1.0 - self.p) * ior2,
                    self.p * transm_color1 + (1.0 - self.p) * transm_color2,
                )),
                None => transm1,
            },
            None => transm2,
        }
    }

    fn reflection(&self) -> Option<Vec3> {
        Some(
            self.mat1.reflection().unwrap_or(Vec3::ZERO) * self.p
                + self.mat2.reflection().unwrap_or(Vec3::ZERO) * (1.0 - self.p),
        )
    }

    fn diffuse(&self) -> Option<Vec3> {
        Some(
            self.mat1.diffuse().unwrap_or(Vec3::ZERO) * self.p
                + self.mat2.diffuse().unwrap_or(Vec3::ZERO) * (1.0 - self.p),
        )
    }

    fn emissive(&self) -> Option<Vec3> {
        Some(
            self.mat1.emissive().unwrap_or(Vec3::ZERO) * self.p
                + self.mat2.emissive().unwrap_or(Vec3::ZERO) * (1.0 - self.p),
        )
    }
}
