use image::Rgb;
use rand::distributions::Distribution;

use crate::{
    camera::Camera,
    hit::{Hit, Hittable, HittableList},
    math::{utils::*, vec::Vec3},
    ray::Ray,
};


pub struct RendererOptions {
    pub samples_per_pixel: u32,
    pub diffuse_depth: u32,
    pub gamma: f64,
}
pub struct Renderer {
    pub camera: Camera,
    pub scene: HittableList,
    pub options: RendererOptions
}

impl Renderer {
    pub fn process_pixel(self: &Renderer, vx: f64, vy: f64) -> Rgb<u8> {
        let distribution_x =
            rand::distributions::Uniform::new(0., 1. / (self.camera.width as f64 - 1.));
        let distribution_y =
            rand::distributions::Uniform::new(0., 1. / (self.camera.height as f64 - 1.));

        let mut rng = rand::thread_rng();
        let v_color = (0..self.options.samples_per_pixel)
            .map(|_| {
                let dvx = distribution_x.sample(&mut rng);
                let dvy = distribution_y.sample(&mut rng);
                Vec3(self.ray_color(&self.camera.ray(vx + dvx, vy + dvy), self.options.diffuse_depth).0)
            })
            .fold(Vec3([0.0, 0.0, 0.0]), |a, b| a + b);

        let v_color = v_color / self.options.samples_per_pixel as f64;

        // Gamma correct 
        let v_color = v_color.0.map(|x| x.powf(1./self.options.gamma));
        // HDR to LDR
        Rgb(v_color.map(|x| (256. * clamp(x)) as u8))
    }

    fn ray_color(&self, ray: &Ray, depth: u32) -> Rgb<f64> {
        if depth == 0 {
            return Rgb([0.0, 0.0, 0.0]);
        }
        let mut rng = rand::thread_rng();

        if let Hit::Hit(record) = self.scene.hit(ray, 0.0..f64::INFINITY) {
            // compute whether the ray in inside or outside the geometry to the ray accordingly
            let bounce_normal = if ray.direction.dot(&record.normal) > 0.0 {
                -record.normal
            } else {
                record.normal
            };

            let bounce_noise =
                Vec3(UnitSphere3::<UnitSphere3PolarMethod>::default().sample(&mut rng));
            let bounce_direction = bounce_normal + bounce_noise;
            let v = self.ray_color(
                &Ray {
                    origin: record.hit_point,
                    direction: bounce_direction,
                },
                depth - 1,
            ).0;
            Rgb((0.5*Vec3(v)).0 )
        } else {
            // No hit, thus, sky 
            let t = 0.5 * (ray.direction.y() + 1.0);
            let v = lerp(t, Vec3::new(0.5, 0.7, 1.0), Vec3::new(1.0, 1.0, 1.0));
            Rgb(v.0)
        }
    }
}
