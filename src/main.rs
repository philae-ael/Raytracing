pub mod camera;
pub mod color;
pub mod hit;
pub mod material;
pub mod math;
pub mod progress;
pub mod ray;
pub mod renderer;

use hit::Sphere;
use image::{ImageBuffer, Rgb, Rgb32FImage};
use math::vec::Vec3;

use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    camera::Camera,
    hit::HittableList,
    material::{Diffuse, Emit, MaterialDescriptor, MaterialId},
    math::utils::*,
    renderer::Renderer,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let out_file = "out.jpg";
    let out_file_old = "out_old.jpg";

    // Ignore failure, we don't care if it doesn't work
    let _ = std::fs::remove_file(out_file_old);
    let _ = std::fs::rename(out_file, out_file_old);

    log::info!("Initialization");
    // Image
    let width = 1920;
    let height = 1080;

    let materials: Vec<MaterialDescriptor> = vec![
        MaterialDescriptor {
            label: Some("Uniform Gray".to_string()),
            material: Box::new(Diffuse {
                color: color::gray(0.5),
            }),
        },
        MaterialDescriptor {
            label: Some("Ground".to_string()),
            material: Box::new(Diffuse {
                color: Rgb([0.2, 0.9, 0.3]),
            }),
        },
        MaterialDescriptor {
            label: Some("Light".to_string()),
            material: Box::new(Emit {
                color: Rgb([2.5, 3.7, 3.9]),
            }),
        },
        MaterialDescriptor {
            label: Some("Sky".to_string()),
            material: Box::new(Emit {
                color: Rgb([0.1, 0.1, 0.4]),
            }),
        },
    ];

    let scene = HittableList(vec![
        Box::new(Sphere {
            label: Some("Sphere".to_string()),
            center: Vec3::new(0.0, 0.0, -1.),
            radius: 0.5,
            material: MaterialId(0),
        }),
        Box::new(Sphere {
            label: Some("Ground".to_string()),
            center: Vec3::new(0.0, -20.5, -1.),
            radius: 20.,
            material: MaterialId(1),
        }),
        Box::new(Sphere {
            label: Some("light".to_string()),
            center: Vec3::new(1.5, 1.0, -2.5),
            radius: 0.6,
            material: MaterialId(2),
        }),
    ]);

    let renderer = Renderer {
        camera: Camera::new(width, height, 2.0, 1.0, Vec3::ZERO),
        scene,
        materials,
        options: renderer::RendererOptions {
            samples_per_pixel: 100,
            diffuse_depth: 50,
            gamma: 2.2,
            world_material: MaterialId(3),
        },
    };

    let mut im: Rgb32FImage = ImageBuffer::new(width, height);
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

    log::info!("Saving HDR image...");
    im.save("out.exr")?;

    let ldr: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(
        width,
        height,
        im.pixels()
            .flat_map(|p| p.0.map(
                    |x| ((u8::MAX as f64) * clamp(x as f64)) as u8)
                )
            .collect(),
    )
    .unwrap();
    log::info!("Saving LDR image...");
    ldr.save(out_file)?;
    Ok(())
}
