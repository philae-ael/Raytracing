use crate::{ray::Ray, shape::local_info};

use super::{texture::Texture, Material, Scattered};

pub struct Emit {
    pub texture: Box<dyn Texture>,
}

impl Material for Emit {
    fn scatter(
        &self,
        _ray: Ray,
        record: &local_info::Full,
        _rng: &mut rand::rngs::ThreadRng,
    ) -> Scattered {
        Scattered {
            ray_out: None,
            albedo: self.texture.color(record.uv),
        }
    }
}
