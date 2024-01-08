pub mod hit;
pub mod math;
pub mod progress;
pub mod ray;

use hit::{Hit, Hittable, Sphere};
use image::{ImageBuffer, Rgb, RgbImage};
use math::utils::*;
use math::vec::Vec3;
use ray::Ray;

use rayon::iter::{ParallelBridge, ParallelIterator};

fn ray_color(ray: &Ray) -> Rgb<f64> {
    let direction = ray.direction;

    let sphere = Sphere {
        center: Vec3::new(0.0, 0.0, -1.0),
        radius: 0.3,
    };

    if let Hit::Hit(record) = sphere.hit(ray, 0.0..f64::INFINITY) {
        Rgb(((record.normal + Vec3::ONES) / 2.0).0)
    } else {
        let t = 0.5 * (direction.y() + 1.0);
        let v = lerp(t, Vec3::new(0.5, 0.7, 1.0), Vec3::new(1.0, 1.0, 1.0));
        Rgb(v.0)
    }
}

pub struct Camera {
    pub viewport_height: f64,
    pub viewport_width: f64,
    pub focal_length: f64,
    pub center: Vec3,
}

impl Camera {
    fn new(viewport_width: f64, viewport_height: f64, focal_length: f64, origin: Vec3) -> Self {
        Self {
            viewport_width,
            viewport_height,
            focal_length,
            center: origin - focal_length * Vec3::Z,
        }
    }
}

pub struct Renderer {
    pub camera: Camera,
    pub origin: Vec3,
}

impl Renderer {
    fn process_pixel(self: &Renderer, vx: f64, vy: f64) -> Rgb<u8> {
        let direction = self.camera.center
            + vx * self.camera.viewport_width * Vec3::X / 2.
            + vy * self.camera.viewport_height * Vec3::Y / 2.;
        let ray = Ray::new(self.origin, direction);
        let color = ray_color(&ray);

        // HDR to LDR
        Rgb(color.0.map(|x| (256. * clamp(x)) as u8))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Initialization");
    // Image
    let width = 1920;
    let height = 1080;
    let aspect_ratio = width as f64 / height as f64;

    let renderer = Renderer {
        camera: Camera::new(2.0 * aspect_ratio, 2.0, 1.0, Vec3::ZERO),
        origin: Vec3::ZERO,
    };

    let mut im: RgbImage = ImageBuffer::new(width, height);
    let progress = progress::Progress::new((width * height) as usize);

    log::info!("Generating image...");
    rayon::scope(|s| {
        s.spawn(|_| {
            while !progress.done() {
                std::thread::sleep(std::time::Duration::from_millis(100));
                progress.print();
            }
            println!();
        });

        let progress = &progress;
        let renderer = &renderer;
        im.enumerate_pixels_mut()
            .par_bridge()
            .for_each(|(x, y, p)| {
                // pixels in the image crate are from left to right, top to bottom
                let vx = 2. * (x as f64 / (width - 1) as f64) - 1.;
                let vy = 1. - 2. * (y as f64 / (height - 1) as f64);
                *p = renderer.process_pixel(vx, vy);
                progress.inc();
            });
    });

    log::info!("Saving image...");
    im.save("out.jpg")?;
    Ok(())
}
