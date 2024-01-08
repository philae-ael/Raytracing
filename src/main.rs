pub mod camera;
pub mod hit;
pub mod math;
pub mod progress;
pub mod ray;
pub mod renderer;

use hit::Sphere;
use image::{ImageBuffer, RgbImage};
use math::vec::Vec3;

use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{camera::Camera, hit::HittableList, renderer::Renderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let out_file = "out.png";
    let out_file_old = "out_old.png";

    // Ignore failure, we don't care if it doesn't work
    let _ = std::fs::remove_file(out_file_old);
    let _ = std::fs::rename(out_file, out_file_old);

    log::info!("Initialization");
    // Image
    let width = 600;
    let height = 400;
    let aspect_ratio = width as f64 / height as f64;

    let sphere = Sphere {
        center: Vec3::new(0.0, 0.0, -1.),
        radius: 0.5,
    };
    let ground = Sphere {
        center: Vec3::new(0.0, -100.5, -1.),
        radius: 100.,
    };
    let scene = HittableList(vec![Box::new(sphere), Box::new(ground)]);
    let renderer = Renderer {
        camera: Camera::new(width, height, 2.0 * aspect_ratio, 2.0, 1.0, Vec3::ZERO),
        scene,
        options: renderer::RendererOptions { samples_per_pixel: 100, diffuse_depth: 50, gamma: 2.2 }
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
    im.save("out.png")?;
    Ok(())
}
