use bytemuck::{Pod, Zeroable};
use rand::distributions::{self, Distribution};

use crate::{
    aggregate::bvh::BVH,
    camera::{Camera, PixelCoord, ViewportCoord},
    color::{self, Luma, Rgb},
    integrators::Integrator,
    material::{MaterialDescriptor, MaterialId},
    math::{
        point::Point,
        quaternion::LookAt,
        stat::RgbSeries,
        vec::{RgbAsVec3Ext, Vec3, Vec3AsRgbExt},
    },
    scene::Scene,
    shape::Shape,
    utils::counter::counter,
};

pub struct RendererOptions {
    pub samples_per_pixel: u32,
    pub allowed_error: Option<f32>,
    pub world_material: MaterialId,
}
pub struct Renderer {
    pub camera: Camera,
    pub objects: Box<dyn Shape>,
    pub lights: Vec<Point>,
    pub options: RendererOptions,

    // TODO: make a pool of materials
    pub materials: Vec<MaterialDescriptor>,
    pub integrator: Box<dyn Integrator>,
}

pub struct RayResult {
    pub normal: Vec3,
    pub position: Point,
    pub albedo: Rgb,
    pub color: Rgb,
    pub z: f32,
    pub ray_depth: f32,
    pub samples_accumulated: u32,
}
pub struct RaySeries {
    pub normal: Vec3,
    pub position: Point,
    pub albedo: Rgb,
    pub color: RgbSeries,
    pub z: f32,
    pub ray_depth: f32,
    pub samples_accumulated: u32,
}

impl RaySeries {
    pub fn resample(self) -> RayResult {
        let RaySeries {
            position,
            normal,
            albedo,
            color,
            z,
            ray_depth,
            samples_accumulated,
        } = self;

        let inv_samples = 1.0 / samples_accumulated as f32;
        RayResult {
            normal: inv_samples * normal,
            position: Point(inv_samples * position.vec()),
            albedo: (inv_samples * albedo.vec()).rgb(),
            color: color.mean(),
            z: inv_samples * z,
            ray_depth: inv_samples * ray_depth,
            samples_accumulated: 1,
        }
    }

    fn add_sample(&mut self, rhs: RayResult) {
        let RayResult {
            normal,
            position,
            albedo,
            color,
            z,
            ray_depth,
            samples_accumulated,
        } = rhs;

        self.color.add_sample(color);
        self.normal += normal;
        self.position = Point(self.position.vec() + position.vec());
        self.albedo = (self.albedo.vec() + albedo.vec()).rgb();
        self.z += z;
        self.ray_depth += ray_depth;
        self.samples_accumulated += samples_accumulated;
    }
}

impl Default for RayResult {
    fn default() -> Self {
        Self {
            normal: color::linear::BLACK.vec(),
            position: Point::ORIGIN,
            albedo: color::linear::BLACK,
            color: color::linear::BLACK,
            z: 0.0,
            ray_depth: 0.0,
            samples_accumulated: 0,
        }
    }
}
impl Default for RaySeries {
    fn default() -> Self {
        Self {
            normal: color::linear::BLACK.vec(),
            position: Point::ORIGIN,
            albedo: color::linear::BLACK,
            color: RgbSeries::default(),
            z: 0.0,
            ray_depth: 0.0,
            samples_accumulated: 0,
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
        let mut ray_series = RaySeries::default();

        for _ in 0..self.options.samples_per_pixel {
            counter!("Samples");
            let dvx = distribution_x.sample(&mut rng);
            let dvy = distribution_y.sample(&mut rng);
            let camera_ray = self.camera.ray(vx + dvx, vy + dvy, &mut rng);

            ray_series.add_sample(self.integrator.ray_cast(self, camera_ray, 0));

            if let Some(allowed_error) = self.options.allowed_error {
                if let Some(_) = ray_series.color.is_precise_enough(allowed_error) {
                    counter!("Adaptative sampling break");
                    break;
                }
            }
        }

        let ray_results = ray_series.resample();

        GenericRenderResult {
            normal: ray_results.normal.rgb(),
            position: ray_results.position.vec().rgb(),
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
    pub allowed_error: Option<f32>,
    pub integrator: Box<dyn Integrator>,
}

impl Into<Renderer> for DefaultRenderer {
    fn into(self) -> Renderer {
        let look_at = Point::new(0.0, 0.0, -1.0);
        let look_from = Point::ORIGIN;
        let look_direction = look_at - look_from;
        let camera = Camera::new(
            self.width,
            self.height,
            f32::to_radians(70.),
            look_direction.length(),
            look_from,
            LookAt {
                direction: look_direction,
                forward: Vec3::NEG_Z,
            }
            .into(),
            0.0,
        );

        let scene = self.scene;

        // let objects = Box::new(scene.objects);
        let objects = Box::new(BVH::from_shapelist(scene.objects));

        Renderer {
            camera,
            objects,
            materials: scene.materials,
            lights: scene.lights,
            options: RendererOptions {
                samples_per_pixel: self.spp,
                world_material: scene.sky_material,
                allowed_error: self.allowed_error,
            },
            integrator: self.integrator,
        }
    }
}
