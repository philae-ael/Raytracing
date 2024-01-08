use rand::prelude::Distribution;

use crate::{
    math::{distributions::*, quaternion::Quat, vec::Vec3},
    ray::Ray,
};

pub struct Camera {
    pub width: u32,
    pub height: u32,
    pub viewport_height: f32,
    pub viewport_width: f32,
    pub focal_length: f32,
    pub origin: Vec3,
    pub rotation: Quat,
    pub aperture: f32,
}

impl Camera {
    pub fn new(
        width: u32,
        height: u32,
        vfov: f32,
        focal_length: f32,
        origin: Vec3,
        rotation: Quat,
        aperture: f32,
    ) -> Self {
        let theta = vfov;
        let h = f32::tan(theta / 2.);

        let aspect_ratio = width as f32 / height as f32;
        Self {
            width,
            height,
            viewport_height: focal_length * h, // From center to top
            viewport_width: focal_length * h * aspect_ratio, // From center to left
            focal_length,
            origin,
            rotation,
            aperture,
        }
    }

    pub fn ray(&self, vx: f32, vy: f32, rng: &mut rand::rngs::ThreadRng) -> Ray {
        let [dx, dy] = UnitBall2.sample(rng);
        let offset = self.aperture / 2.0
            * Vec3 {
                x: dx,
                y: dy,
                z: 0.0,
            };
        let center = self.origin - self.focal_length * Vec3::Z;

        let direction = center - (self.origin + offset)
            + vx * self.viewport_width * Vec3::X
            + vy * self.viewport_height * Vec3::Y;
        Ray::new(
            self.origin + self.rotation.mul_vec3(offset),
            self.rotation.mul_vec3(direction),
        )
    }
}
