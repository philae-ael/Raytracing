use image::Rgb;
use rand::distributions::Distribution;

use crate::{
    camera::Camera,
    color,
    hit::{Hit, HitRecord, Hittable, HittableList},
    material::{MaterialDescriptor, MaterialId},
    math::vec::{RgbAsVec3Ext, Vec3, Vec3AsRgbExt},
    ray::Ray,
};

pub struct RendererOptions {
    pub samples_per_pixel: u32,
    pub diffuse_depth: u32,
    pub gamma: f64,
    pub world_material: MaterialId,
}
pub struct Renderer {
    pub camera: Camera,
    pub scene: HittableList,
    pub options: RendererOptions,

    // TODO: make a pool of materials
    pub materials: Vec<MaterialDescriptor>,
}

struct RayResult {
    normal: Vec3,
    albedo: Rgb<f64>,
    color: Rgb<f64>,
    depth: f64,
}

pub struct RenderResult {
    pub normal: Rgb<f64>,
    pub ddepth: Rgb<f64>,
    pub albedo: Rgb<f64>,
    pub color: Rgb<f64>,
    pub depth: f64,
}

impl Renderer {
    pub fn process_pixel(self: &Renderer, vx: f64, vy: f64) -> RenderResult {
        let pixel_width = 1. / (self.camera.width as f64 - 1.);
        let pixel_height = 1. / (self.camera.height as f64 - 1.);
        let distribution_x = rand::distributions::Uniform::new(0., pixel_width);
        let distribution_y = rand::distributions::Uniform::new(0., pixel_height);

        let mut rng = rand::thread_rng();
        let mut max_t = 0.0;
        let mut min_t = 1e100;
        let ray_results = {
            let ray_results_acc = (0..self.options.samples_per_pixel)
                .map(|_| {
                    let dvx = distribution_x.sample(&mut rng);
                    let dvy = distribution_y.sample(&mut rng);
                    let ray_result = self.throw_ray(
                        &self.camera.ray(vx + dvx, vy + dvy, &mut rng),
                        self.options.diffuse_depth,
                    );

                    max_t = f64::max(max_t, ray_result.depth);
                    min_t = f64::min(min_t, ray_result.depth);

                    ray_result
                })
                .fold(
                    // Accumulate pixels
                    RayResult {
                        normal: Vec3::ZERO,
                        color: color::BLACK,
                        depth: f64::INFINITY,
                        albedo: color::BLACK,
                    },
                    |RayResult {
                         normal: normal1,
                         color: color1,
                         depth: depth1,
                         albedo: albedo1,
                     },
                     RayResult {
                         normal: normal2,
                         color: color2,
                         depth: depth2,
                         albedo: albedo2,
                     }| RayResult {
                        normal: normal1 + normal2,
                        color: (color1.vec() + color2.vec()).rgb(),
                        depth: depth1 + depth2,
                        albedo: (albedo1.vec() + albedo2.vec()).rgb(),
                    },
                );

            // Then renormalize them
            let samples = self.options.samples_per_pixel as f64;
            RayResult {
                normal: ray_results_acc.normal / samples,
                color: (ray_results_acc.color.vec() / samples).rgb(),
                depth: ray_results_acc.depth / samples,
                albedo: (ray_results_acc.albedo.vec() / samples).rgb(),
            }
        };

        if max_t == f64::INFINITY {
            max_t = 0.0;
            min_t = 0.0
        }
        let ddepth = Vec3([min_t, max_t, 0.0]).rgb();
        // Gamma correct
        let color = Rgb(ray_results.color.0.map(|x| x.powf(1. / self.options.gamma)));

        RenderResult {
            normal: ray_results.normal.rgb(),
            ddepth,
            color,
            albedo: ray_results.albedo,
            depth: ray_results.depth,
        }
    }

    fn throw_ray(&self, ray: &Ray, depth: u32) -> RayResult {
        if depth == 0 {
            return RayResult {
                normal: Vec3::ZERO,
                color: color::BLACK,
                depth: f64::INFINITY,
                albedo: color::BLACK,
            };
        }
        let mut rng = rand::thread_rng();

        if let Hit::Hit(record) = self.scene.hit(ray, 0.01..f64::INFINITY) {
            let material = &self.materials[record.material.0].material;
            let scattered = material.scatter(ray, &record, &mut rng);

            let color = if let Some(ray_out) = scattered.ray_out {
                self.throw_ray(&ray_out, depth - 1).color
            } else {
                color::WHITE
            };
            let color = color::mix(color::MixMode::Mul, color, scattered.albedo);
            RayResult {
                normal: record.normal,
                color,
                depth: record.t,
                albedo: scattered.albedo,
            }
        } else {
            let material = &self.materials[self.options.world_material.0].material;
            let record = HitRecord {
                t: f64::INFINITY,
                hit_point: ray.origin,
                normal: -ray.direction,
                material: self.options.world_material,
            };
            let scattered = material.scatter(ray, &record, &mut rng);
            RayResult {
                normal: Vec3::ZERO,
                color: scattered.albedo,
                depth: f64::INFINITY,
                albedo: color::BLACK,
            }
        }
    }
}
