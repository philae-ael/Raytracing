use image::Rgb;
use rand::distributions::Distribution;

use crate::{
    camera::Camera,
    color::{self, Color},
    hit::{Hit, HitRecord, Hittable, HittableList},
    material::{MaterialDescriptor, MaterialId},
    math::vec::Vec3,
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

impl Renderer {
    pub fn process_pixel(self: &Renderer, vx: f64, vy: f64) -> Rgb<f32> {
        let distribution_x =
            rand::distributions::Uniform::new(0., 1. / (self.camera.width as f64 - 1.));
        let distribution_y =
            rand::distributions::Uniform::new(0., 1. / (self.camera.height as f64 - 1.));

        let mut rng = rand::thread_rng();
        let v_color = (0..self.options.samples_per_pixel)
            .map(|_| {
                let dvx = distribution_x.sample(&mut rng);
                let dvy = distribution_y.sample(&mut rng);
                Vec3(
                    self.ray_color(
                        &self.camera.ray(vx + dvx, vy + dvy),
                        self.options.diffuse_depth,
                    )
                    .0,
                )
            })
            .fold(Vec3([0.0, 0.0, 0.0]), |a, b| a + b);

        let v_color = v_color / self.options.samples_per_pixel as f64;

        // Gamma correct
        let color = Rgb(v_color.0.map(|x| x.powf(1. / self.options.gamma)));
        Rgb(color.0.map(|x| x as f32))
    }

    fn ray_color(&self, ray: &Ray, depth: u32) -> Color {
        if depth == 0 {
            return color::BLACK;
        }
        let mut rng = rand::thread_rng();

        if let Hit::Hit(record) = self.scene.hit(ray, 0.01..f64::INFINITY) {
            let material = &self.materials[record.material.0].material;
            let scattered = material.scatter(ray, &record, &mut rng);

            let color = if let Some(ray_out) = scattered.ray_out {
                self.ray_color(&ray_out, depth - 1)
            } else {
                color::WHITE
            };
            color::mix(color::MixMode::Mul, color, scattered.absorption)
        } else {
            let material = &self.materials[self.options.world_material.0].material;
            let record = HitRecord {
                t: f64::INFINITY,
                hit_point: ray.origin,
                normal: -ray.direction,
                material: self.options.world_material,
            };
            let scattered = material.scatter(ray, &record, &mut rng);
            scattered.absorption
        }
    }
}
