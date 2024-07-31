use glam::Vec2;
use rand::prelude::Distribution;

use crate::{
    math::{distributions::UnitBall2, point::Point, quaternion::Quat, vec::Vec3},
    ray::Ray,
    Ctx,
};

pub struct Camera {
    /// Aperture is the diameter of the the opening of the camera.\
    /// Higher aperture means more light incomming but a more blurry image.
    pub aperture: f32,

    /// The focal length is distance between the sensor and the lens, in world unit.
    /// It determines the focus point.
    pub focal_length: f32,

    /// width of the sensor, in pixel
    pub width: u32,
    /// height of the sensor, in pixel
    pub height: u32,

    /// half of the height of the sensor, in world unit
    pub viewport_half_height: f32,
    /// half of the width of the sensor, in world unit
    pub viewport_half_width: f32,

    pub center_of_lens: Point,

    // should use scene transformation and assume camera is always facing the +Z direction
    pub rotation: Quat,
}

impl Camera {
    pub fn new(
        width: u32,
        height: u32,
        vfov: f32,
        focal_length: f32,
        center_of_lens: Point,
        rotation: Quat,
        aperture: f32,
    ) -> Self {
        let half_height_factor = f32::tan(vfov / 2.);

        let aspect_ratio = width as f32 / height as f32;
        Self {
            width,
            height,
            viewport_half_height: focal_length * half_height_factor, // From center to top
            viewport_half_width: focal_length * half_height_factor * aspect_ratio, // From center to left
            focal_length,
            center_of_lens,
            rotation,
            aperture,
        }
    }

    /// Generate a ray outgoing from the given [ViewportCoord]
    ///
    /// Simulate aperture, focal length stochastically
    pub fn ray(&self, ctx: &mut Ctx, coords: Vec2) -> Ray {
        let vcoords = ViewportCoord::from_pixel_coord(self, coords);
        let center_of_sensor = self.center_of_lens + self.focal_length * Vec3::Z;

        // from the sensor
        let ray_origin = center_of_sensor
            + vcoords.vx * self.viewport_half_width * Vec3::X
            + vcoords.vy * self.viewport_half_height * Vec3::Y;

        // to the lens
        let [dx, dy] = UnitBall2.sample(&mut ctx.rng);
        let offset = self.aperture / 2.0
            * Vec3 {
                x: dx,
                y: dy,
                z: 0.0,
            };
        let ray_dst = self.center_of_lens + offset;

        Ray::new(
            self.center_of_lens,
            self.rotation.mul_vec3(ray_dst - ray_origin).normalize(),
        )
    }
}

/// Represent a coordinate in the viewport space.
///
/// The viewport is mapped to the range $\left[-1, -1\right]$ for both `vx` and `vy`.
///
/// $\left(-1, -1\right)$ is the top left corner
#[derive(Debug, Clone, Copy)]
pub struct ViewportCoord {
    pub vx: f32,
    pub vy: f32,
}

impl ViewportCoord {
    // Convert a coordinate in pixel space into viewport space
    pub fn from_pixel_coord(camera: &Camera, coords: Vec2) -> Self {
        Self {
            vx: 2. * (coords.x / camera.width as f32) - 1.,
            vy: 2. * (coords.y / camera.height as f32) - 1.,
        }
    }
}
