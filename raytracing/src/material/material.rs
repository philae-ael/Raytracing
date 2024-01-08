use glam::Vec3;

use crate::{ray::Ray, shape::local_info, color::Rgb};

pub trait Material {
    fn scatter(
        &self,
        ray: Ray,
        record: &local_info::Full,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Scattered;

    fn transmission(&self) -> Option<(f32, Vec3)> {
        None
    }
    fn reflection(&self) -> Option<Vec3> {
        None
    }

    fn diffuse(&self) -> Option<Vec3> {
        None
    }

    fn emissive(&self) -> Option<Vec3> {
        None
    }
}

pub struct Scattered {
    pub albedo: Rgb,
    pub ray_out: Option<Ray>,
}

pub struct MaterialDescriptor {
    pub label: Option<String>,
    pub material: Box<dyn Material + Sync + Send>,
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
