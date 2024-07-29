use crate::{math::vec::RgbAsVec3Ext, ray::Ray, shape::local_info, Rng};

use super::{texture::Texture, Material, Scattered};

pub struct Emit {
    pub texture: Box<dyn Texture>,
}

impl Material for Emit {
    fn scatter(&self, _ray: Ray, record: &local_info::Full, _rng: &mut Rng) -> Scattered {
        Scattered {
            ray_out: None,
            albedo: self.texture.color(record.uv),
        }
    }
    fn emissive(&self) -> Option<glam::Vec3> {
        Some(self.texture.color([0.0, 0.0]).vec())
    }
}
