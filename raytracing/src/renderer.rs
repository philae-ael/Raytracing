use std::ops::Add;

use bytemuck::{Pod, Zeroable};
use rand::distributions::{self, Distribution};

use crate::{
    aggregate::shapelist::ShapeList,
    camera::{Camera, PixelCoord, ViewportCoord},
    color::{self, Luma, Rgb},
    integrators::Integrator,
    material::{texture::Uniform, Emit, MaterialDescriptor, MaterialId},
    math::{
        quaternion::LookAt,
        vec::{RgbAsVec3Ext, Vec3, Vec3AsRgbExt},
    },
    scene::Scene,
};

pub struct RendererOptions {
    pub samples_per_pixel: u32,
    pub world_material: MaterialId,
}
pub struct Renderer {
    pub camera: Camera,
    pub objects: ShapeList,
    pub lights: Vec<Vec3>,
    pub options: RendererOptions,

    // TODO: make a pool of materials
    pub materials: Vec<MaterialDescriptor>,
    pub integrator: Box<dyn Integrator>,
}

pub struct RayResult {
    pub normal: Vec3,
    pub position: Vec3,
    pub albedo: Rgb,
    pub color: Rgb,
    pub z: f32,
    pub ray_depth: f32,
    pub samples_accumulated: u32,
}

impl RayResult {
    pub fn resample(self) -> Self {
        let RayResult {
            position,
            normal,
            albedo,
            color,
            z,
            ray_depth,
            samples_accumulated,
        } = self;

        let inv_samples = 1.0 / samples_accumulated as f32;
        Self {
            normal: inv_samples * normal,
            position: inv_samples * position,
            albedo: (inv_samples * albedo.vec()).rgb(),
            color: (inv_samples * color.vec()).rgb(),
            z: inv_samples * z,
            ray_depth: inv_samples * ray_depth,
            samples_accumulated: 1,
        }
    }
}

impl Default for RayResult {
    fn default() -> Self {
        Self {
            normal: color::linear::BLACK.vec(),
            position: Vec3::ZERO,
            albedo: color::linear::BLACK,
            color: color::linear::BLACK,
            z: 0.0,
            ray_depth: 0.0,
            samples_accumulated: 0,
        }
    }
}

impl Add for RayResult {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let RayResult {
            normal: normal1,
            position: position1,
            albedo: albedo1,
            color: color1,
            z: z1,
            ray_depth: ray_depth1,
            samples_accumulated: samples_accumulated1,
        } = self;

        let RayResult {
            normal: normal2,
            position: position2,
            albedo: albedo2,
            color: color2,
            z: z2,
            ray_depth: ray_depth2,
            samples_accumulated: samples_accumulated2,
        } = rhs;

        RayResult {
            normal: normal1 + normal2,
            position: position1 + position2,
            albedo: (albedo1.vec() + albedo2.vec()).rgb(),
            color: (color1.vec() + color2.vec()).rgb(),
            z: z1 + z2,
            ray_depth: ray_depth1 + ray_depth2,
            samples_accumulated: samples_accumulated1 + samples_accumulated2,
        }
    }
}

pub enum Channel<RgbStorage, LumaStorage> {
    Color(RgbStorage),
    Normal(RgbStorage),
    Position(RgbStorage),
    Albedo(RgbStorage),
    Z(LumaStorage),
    RayDepth(LumaStorage),
}
const CHANNEL_COUNT: usize = 6;

#[repr(C)]
pub struct GenericRenderResult<RgbStorage, LumaStorage> {
    pub color: RgbStorage,
    pub normal: RgbStorage,
    pub position: RgbStorage,
    pub albedo: RgbStorage,
    pub z: LumaStorage,
    pub ray_depth: LumaStorage,
}

impl<RgbStorage, LumaStorage> GenericRenderResult<RgbStorage, LumaStorage> {
    pub fn as_ref(&self) -> GenericRenderResult<&RgbStorage, &LumaStorage> {
        GenericRenderResult {
            color: &self.color,
            normal: &self.normal,
            position: &&self.position,
            albedo: &self.albedo,
            z: &self.z,
            ray_depth: &self.ray_depth,
        }
    }
}

pub type PixelRenderResult = GenericRenderResult<Rgb, Luma>;

impl<RgbStorage, LumaStorage> IntoIterator for GenericRenderResult<RgbStorage, LumaStorage> {
    type Item = Channel<RgbStorage, LumaStorage>;

    type IntoIter = <[Channel<RgbStorage, LumaStorage>; CHANNEL_COUNT] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        [
            Channel::Color(self.color),
            Channel::Albedo(self.albedo),
            Channel::Position(self.position),
            Channel::Normal(self.normal),
            Channel::Z(self.z),
            Channel::RayDepth(self.ray_depth),
        ]
        .into_iter()
    }
}

impl Clone for PixelRenderResult {
    fn clone(&self) -> Self {
        Self {
            color: self.color.clone(),
            position: self.position.clone(),
            normal: self.normal.clone(),
            albedo: self.albedo.clone(),
            z: self.z.clone(),
            ray_depth: self.ray_depth.clone(),
        }
    }
}
impl std::marker::Copy for PixelRenderResult {}

/// SAFETY:  needed because we can't derive Pod and Zeroable for all GenericRenderResult
/// - PixelRenderResult is Zeroable,
/// - all bits patterns are valid,
/// - all his fields are pods,
/// - it is repr(C),
/// - there is no interior mutability
unsafe impl Pod for PixelRenderResult {}

/// SAFETY:  PixelRenderResult is inhabited and the all-zero pattern is allowed
unsafe impl Zeroable for PixelRenderResult {}

impl Renderer {
    pub fn process_pixel(self: &Renderer, coords: PixelCoord) -> PixelRenderResult {
        let ViewportCoord { vx, vy } = ViewportCoord::from_pixel_coord(&self.camera, coords);
        let pixel_width = 1. / (self.camera.width as f32 - 1.);
        let pixel_height = 1. / (self.camera.height as f32 - 1.);
        let distribution_x = distributions::Uniform::new(-pixel_width / 2., pixel_width / 2.);
        let distribution_y = distributions::Uniform::new(-pixel_height / 2., pixel_height / 2.);

        let mut rng = rand::thread_rng();
        let ray_results = (0..self.options.samples_per_pixel)
            .map(|_| {
                let dvx = distribution_x.sample(&mut rng);
                let dvy = distribution_y.sample(&mut rng);
                let camera_ray = self.camera.ray(vx + dvx, vy + dvy, &mut rng);
                self.integrator.ray_cast(self, camera_ray, 0)
            })
            .fold(RayResult::default(), RayResult::add)
            .resample();

        GenericRenderResult {
            normal: ray_results.normal.rgb(),
            position: ray_results.position.rgb(),
            color: ray_results.color,
            albedo: ray_results.albedo,
            z: Luma(ray_results.z),
            ray_depth: Luma(ray_results.ray_depth),
        }
    }
}

pub struct DefaultRenderer {
    pub width: u32,
    pub height: u32,
    pub spp: u32,
    pub scene: Scene,
    pub integrator: Box<dyn Integrator>,
}

impl Into<Renderer> for DefaultRenderer {
    fn into(self) -> Renderer {
        let look_at = Vec3::NEG_Z;
        let look_from = Vec3::ZERO;
        let look_direction = look_at - look_from;
        let camera = Camera::new(
            self.width,
            self.height,
            f32::to_radians(90.),
            look_direction.length(),
            look_from,
            LookAt {
                direction: look_direction,
                forward: Vec3::NEG_Z,
            }
            .into(),
            0.0,
        );

        let mut scene = self.scene;

        let sky_mat = scene.insert_material(MaterialDescriptor {
            label: Some("Sky".to_owned()),
            material: Box::new(Emit {
                texture: Box::new(Uniform(Vec3::splat(0.3).rgb())),
            }),
        });

        Renderer {
            camera,
            objects: scene.objects,
            materials: scene.materials,
            lights: scene.lights,
            options: RendererOptions {
                samples_per_pixel: self.spp,
                world_material: sky_mat,
            },
            integrator: self.integrator,
        }
    }
}
