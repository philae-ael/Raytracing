use image::Rgb;

use crate::{ray::Ray, shape::local_info};

pub trait Material {
    fn scatter(
        &self,
        ray: Ray,
        record: &local_info::Full,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Scattered;
}

pub struct Scattered {
    pub albedo: Rgb<f32>,
    pub ray_out: Option<Ray>,
}

pub struct MaterialDescriptor {
    pub label: Option<String>,
    pub material: Box<dyn Material + Sync>,
}

impl std::fmt::Debug for MaterialDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaterialDescriptor")
            .field("label", &self.label)
            .field("material", &"<material>")
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialId(pub usize);
