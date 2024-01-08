use image::{buffer::EnumeratePixelsMut, Luma, Rgb, Rgb32FImage};
use rand::distributions::Distribution;
use rayon::prelude::{ParallelBridge, ParallelIterator};

use crate::{
    camera::Camera,
    color,
    hit::{Hit, HitRecord, Hittable, HittableList},
    material::{MaterialDescriptor, MaterialId},
    math::vec::{RgbAsVec3Ext, Vec3, Vec3AsRgbExt},
    progress,
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
    pub albedo: Rgb<f64>,
    pub color: Rgb<f64>,
    pub depth: f64,
}

pub struct OutputBuffers {
    pub color: Rgb32FImage,
    pub normal: Rgb32FImage,
    pub albedo: Rgb32FImage,
    pub depth: image::ImageBuffer<Luma<f32>, Vec<f32>>,
}

impl OutputBuffers {
    fn iter(&'_ mut self) -> OutputBuffersIterator<'_, f32> {
        OutputBuffersIterator {
            color: self.color.enumerate_pixels_mut(),
            normal: self.normal.enumerate_pixels_mut(),
            albedo: self.albedo.enumerate_pixels_mut(),
            depth: self.depth.enumerate_pixels_mut(),
        }
    }
}

struct OutputBuffersIterator<'a, T>
where
    Rgb<T>: image::Pixel,
    Luma<T>: image::Pixel,
{
    color: EnumeratePixelsMut<'a, Rgb<T>>,
    normal: EnumeratePixelsMut<'a, Rgb<T>>,
    albedo: EnumeratePixelsMut<'a, Rgb<T>>,
    depth: EnumeratePixelsMut<'a, Luma<T>>,
}
struct OutputBufferProxy<'a, T>
where
    Rgb<T>: image::Pixel,
    Luma<T>: image::Pixel,
{
    pub x: u32,
    pub y: u32,
    color: &'a mut Rgb<T>,
    normal: &'a mut Rgb<T>,
    albedo: &'a mut Rgb<T>,
    depth: &'a mut Luma<T>,
}

impl<'a, T> Iterator for OutputBuffersIterator<'a, T>
where
    Rgb<T>: image::Pixel,
    Luma<T>: image::Pixel,
{
    type Item = OutputBufferProxy<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (x, y, normal) = self.normal.next()?;
        let albedo = self.albedo.next()?.2;
        let color = self.color.next()?.2;
        let depth = self.depth.next()?.2;
        Some(OutputBufferProxy {
            x,
            y,
            normal,
            albedo,
            color,
            depth,
        })
    }
}

impl Renderer {
    pub fn process_pixel(self: &Renderer, vx: f64, vy: f64) -> RenderResult {
        let pixel_width = 1. / (self.camera.width as f64 - 1.);
        let pixel_height = 1. / (self.camera.height as f64 - 1.);
        let distribution_x = rand::distributions::Uniform::new(0., pixel_width);
        let distribution_y = rand::distributions::Uniform::new(0., pixel_height);

        let mut rng = rand::thread_rng();
        let ray_results = {
            let ray_results_acc = (0..self.options.samples_per_pixel)
                .map(|_| {
                    let dvx = distribution_x.sample(&mut rng);
                    let dvy = distribution_y.sample(&mut rng);
                    self.throw_ray(
                        &self.camera.ray(vx + dvx, vy + dvy, &mut rng),
                        self.options.diffuse_depth,
                    )
                })
                .fold(
                    // Accumulate pixels
                    RayResult {
                        normal: Vec3::ZERO,
                        color: color::BLACK,
                        depth: 0.0,
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

        // Gamma correct
        let color = Rgb(ray_results.color.0.map(|x| x.powf(1. / self.options.gamma)));

        RenderResult {
            normal: (0.5*(1.0*Vec3::ONES + ray_results.normal)).rgb(),
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
                depth: -1.0,
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
                t: 0.0,
                hit_point: ray.origin,
                normal: -ray.direction,
                material: self.options.world_material,
            };
            let scattered = material.scatter(ray, &record, &mut rng);
            RayResult {
                normal: Vec3::ZERO,
                color: scattered.albedo,
                depth: 0.0,
                albedo: color::BLACK,
            }
        }
    }

    pub fn run_scene(&self, output_buffer: &mut OutputBuffers) {
        let progress = progress::Progress::new((self.camera.width * self.camera.height) as usize);

        log::info!("Generating image...");
        rayon::scope(|s| {
            s.spawn(|_| {
                while !progress.done() {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    progress.print();
                }
                println!();
            });

            output_buffer.iter().par_bridge().for_each(|p| {
                // pixels in the image crate are from left to right, top to bottom
                let vx = 2. * (p.x as f64 / (self.camera.width - 1) as f64) - 1.;
                let vy = 1. - 2. * (p.y as f64 / (self.camera.height - 1) as f64);
                let render_result = self.process_pixel(vx, vy);
                *p.color = color::convert_lossy(render_result.color);
                *p.normal = color::convert_lossy(render_result.normal);
                *p.albedo = color::convert_lossy(render_result.albedo);
                *p.depth = Luma([render_result.depth as f32]);
                progress.inc();
            });
        });
    }
}
