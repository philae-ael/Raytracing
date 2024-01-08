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
        rng: &mut rand::rngs::ThreadRng,
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
}
