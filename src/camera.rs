use crate::{math::vec::Vec3, ray::Ray};

pub struct Camera {
    pub width: u32,
    pub height: u32,
    pub viewport_height: f64,
    pub viewport_width: f64,
    pub focal_length: f64,
    pub center: Vec3,
    pub origin: Vec3,
}

impl Camera {
    pub fn new(
        width: u32,
        height: u32,
        viewport_width: f64,
        viewport_height: f64,
        focal_length: f64,
        origin: Vec3,
    ) -> Self {
        Self {
            width,
            height,
            viewport_width,
            viewport_height,
            focal_length,
            center: origin - focal_length * Vec3::Z,
            origin,
        }
    }

    pub fn ray(&self, vx: f64, vy: f64) -> Ray {
        let direction = self.center
            + vx * self.viewport_width * Vec3::X / 2.
            + vy * self.viewport_height * Vec3::Y / 2.;
        Ray::new(self.origin, direction)
    }
}
