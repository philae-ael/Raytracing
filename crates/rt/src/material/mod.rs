mod dielectric;
mod diffuse;
mod emit;
mod gooch;
mod metal;
mod mix;
mod phong;
pub mod texture;

pub use dielectric::Dielectric;
pub use diffuse::Diffuse;
pub use emit::Emit;
pub use gooch::Gooch;
pub use metal::Metal;
pub use mix::MixMaterial;

use glam::Vec3;

use crate::{color::Rgb, math::point::Point, ray::Ray, shape::local_info};

pub trait Material: Sync + Send {
    fn scatter(
        &self,
        ray: Ray,
        record: &local_info::Full,
        rng: &mut rand::rngs::StdRng,
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
    pub material: Box<dyn Material>,
}

impl std::fmt::Debug for MaterialDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaterialDescriptor")
            .field("label", &self.label)
            .field("material", &"<material>")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct LightDescriptor {
    pub label: Option<String>,
    pub light_pos: Point,
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialId(pub usize);
