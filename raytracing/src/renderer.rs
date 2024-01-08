use std::ops::Add;

use bytemuck::{Pod, Zeroable};
use image::Rgb;
use rand::distributions::Distribution;

use crate::{
    aggregate::shapelist::ShapeList,
    camera::{Camera, PixelCoord, ViewportCoord},
    color::{self, BLACK},
    material::{texture::Uniform, Emit, MaterialDescriptor, MaterialId},
    math::{
        distributions::sphere_uv_from_direction,
        quaternion::LookAt,
        vec::{RgbAsVec3Ext, Vec3, Vec3AsRgbExt},
    },
    ray::Ray,
    scene::Scene,
    shape::{local_info, IntersectionResult, Shape},
};

pub struct RendererOptions {
    pub samples_per_pixel: u32,
    pub diffuse_depth: u32,
    pub gamma: f32,
    pub world_material: MaterialId,
}
pub struct Renderer {
    pub camera: Camera,
    pub objects: ShapeList,
    pub options: RendererOptions,

    // TODO: make a pool of materials
    pub materials: Vec<MaterialDescriptor>,
}

struct RayResult {
    normal: Vec3,
    albedo: Rgb<f32>,
    color: Rgb<f32>,
    z: f32,
    ray_depth: f32,
    samples_accumulated: u32,
}

impl RayResult {
    pub fn resample(self) -> Self {
        let RayResult {
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
            normal: BLACK.vec(),
            albedo: BLACK,
            color: BLACK,
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
            albedo: albedo1,
            color: color1,
            z: z1,
            ray_depth: ray_depth1,
            samples_accumulated: samples_accumulated1,
        } = self;

        let RayResult {
            normal: normal2,
            albedo: albedo2,
            color: color2,
            z: z2,
            ray_depth: ray_depth2,
            samples_accumulated: samples_accumulated2,
        } = rhs;

        RayResult {
            normal: normal1 + normal2,
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
    Albedo(RgbStorage),
    Z(LumaStorage),
    RayDepth(LumaStorage),
}
const CHANNEL_COUNT: usize = 5;

#[repr(C)]
pub struct GenericRenderResult<RgbStorage, LumaStorage> {
    pub color: RgbStorage,
    pub normal: RgbStorage,
    pub albedo: RgbStorage,
    pub z: LumaStorage,
    pub ray_depth: LumaStorage,
}

impl<RgbStorage, LumaStorage> GenericRenderResult<RgbStorage, LumaStorage> {
    pub fn as_ref(&self) -> GenericRenderResult<&RgbStorage, &LumaStorage> {
        GenericRenderResult {
            color: &self.color,
            normal: &self.normal,
            albedo: &self.albedo,
            z: &self.z,
            ray_depth: &self.ray_depth,
        }
    }
}

pub type PixelRenderResult = GenericRenderResult<[f32; 3], f32>;

impl<RgbStorage, LumaStorage> IntoIterator for GenericRenderResult<RgbStorage, LumaStorage> {
    type Item = Channel<RgbStorage, LumaStorage>;

    type IntoIter = <[Channel<RgbStorage, LumaStorage>; CHANNEL_COUNT] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        [
            Channel::Color(self.color),
            Channel::Albedo(self.albedo),
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
        let distribution_x = rand::distributions::Uniform::new(0., pixel_width);
        let distribution_y = rand::distributions::Uniform::new(0., pixel_height);

        let mut rng = rand::thread_rng();
        let ray_results = (0..self.options.samples_per_pixel)
            .map(|_| {
                let dvx = distribution_x.sample(&mut rng);
                let dvy = distribution_y.sample(&mut rng);
                self.throw_ray(
                    self.camera.ray(vx + dvx, vy + dvy, &mut rng),
                    self.options.diffuse_depth,
                )
            })
            .fold(RayResult::default(), RayResult::add)
            .resample();

        // Gamma correct
        let color = Rgb(ray_results.color.0.map(|x| x.powf(1. / self.options.gamma)));

        GenericRenderResult {
            normal: ray_results.normal.to_array(),
            color: color.0,
            albedo: ray_results.albedo.0,
            z: ray_results.z,
            ray_depth: ray_results.ray_depth,
        }
    }

    fn throw_ray(&self, ray: Ray, depth: u32) -> RayResult {
        let mut rng = rand::thread_rng();
        if depth == 0 {
            return RayResult::default();
        }

        // Prevent auto intersection
        let ray = Ray::new_with_range(ray.origin, ray.direction, 0.01..ray.bounds.1);

        if let IntersectionResult::Instersection(record) = self.objects.intersection_full(ray) {
            // On material hit
            let material = &self.materials[record.local_info.material.0].material;
            let scattered = material.scatter(ray, &record.local_info, &mut rng);

            let (color, ray_depth) = if let Some(ray_out) = scattered.ray_out {
                let ray_result = self.throw_ray(ray_out, depth - 1);
                (ray_result.color, ray_result.ray_depth)
            } else {
                (color::WHITE, 0.0)
            };

            let color = (color.vec() * scattered.albedo.vec()).rgb();

            RayResult {
                normal: record.local_info.normal,
                color,
                z: record.t,
                albedo: scattered.albedo,
                ray_depth: ray_depth + 1.0,
                samples_accumulated: 1,
            }
        } else {
            // Sky
            let material = &self.materials[self.options.world_material.0].material;
            let record = local_info::Full {
                pos: ray.origin,
                normal: -ray.direction,
                material: self.options.world_material,
                uv: sphere_uv_from_direction(-ray.direction),
            };
            let scattered = material.scatter(ray, &record, &mut rng);
            RayResult {
                normal: Vec3::ZERO,
                albedo: color::BLACK,
                color: scattered.albedo,
                z: 0.0,
                ray_depth: 0.0,
                samples_accumulated: 1,
            }
        }
    }
}

pub struct DefaultRenderer {
    pub width: u32,
    pub height: u32,
    pub spp: u32,
    pub scene: Scene,
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
                texture: Box::new(Uniform(Rgb([0.2, 0.2, 0.2]))),
            }),
        });

        Renderer {
            camera,
            objects: scene.objects,
            materials: scene.materials,
            options: RendererOptions {
                samples_per_pixel: self.spp,
                diffuse_depth: 20,
                gamma: 1.0,
                world_material: sky_mat,
            },
        }
    }
}
