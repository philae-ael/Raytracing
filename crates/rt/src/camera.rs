use rand::{distributions::Uniform, prelude::Distribution};

use crate::{
    math::{distributions::*, point::Point, quaternion::Quat, vec::Vec3},
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

    /// width of the sensor, in world unit
    pub viewport_height: f32,
    /// width of the sensor, in world unit
    pub viewport_width: f32,

    //
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
        center_of_length: Point,
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
            center_of_lens: center_of_length,
            rotation,
            aperture,
        }
    }

    /// Generate a ray outgoing from the given [ViewportCoord]
    ///
    /// Simulate aperture and focal length stochastically
    pub fn ray(&self, ctx: &mut Ctx, coords: ViewportCoord) -> Ray {
        let center_of_sensor = self.center_of_lens + self.focal_length * Vec3::Z;

        // from the sensor
        let ray_origin = center_of_sensor
            + coords.vx * self.viewport_width * Vec3::X
            + coords.vy * self.viewport_height * Vec3::Y;

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
            self.rotation.mul_vec3(ray_dst - ray_origin),
        )
    }
}

/// Represent a coordinate in the pixel space.
///
/// The viewport is mapped to the range $\left[0, 1\right]$ for both `x` and `y`.
///
/// $\left\(0, 0\right)$ is the top left corner.
#[derive(Debug, Clone, Copy)]
pub struct PixelCoord {
    pub x: f32,
    pub y: f32,
}

impl PixelCoord {
    /// Sample a point around the pixel located at `coords`
    ///
    /// Given a pixel coordinate (x, y), the sample is taken uniformely in
    /// $\left[x, x+1\right[ \times \left[y, x+1\right[$
    pub fn sample_around(ctx: &mut Ctx, x: u32, y: u32) -> PixelCoord {
        let uniform = Uniform::new(0., 1.);
        let dx = uniform.sample(&mut ctx.rng);
        let dy = uniform.sample(&mut ctx.rng);
        PixelCoord {
            x: x as f32 + dx,
            y: y as f32 + dy,
        }
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
    pub fn from_pixel_coord(camera: &Camera, coord: PixelCoord) -> Self {
        Self {
            vx: 2. * (coord.x / (camera.width - 1) as f32) - 1.,
            vy: 2. * (coord.y / (camera.height - 1) as f32) - 1.,
        }
    }
}
