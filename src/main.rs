pub mod camera;
pub mod color;
pub mod hit;
pub mod material;
pub mod math;
pub mod progress;
pub mod ray;
pub mod renderer;

use hit::Sphere;
use image::{buffer::ConvertBuffer, ImageBuffer, Rgb};
use math::vec::Vec3;

use crate::{
    camera::Camera,
    hit::HittableList,
    material::{texture, Dielectric, Diffuse, Emit, MaterialDescriptor, MaterialId, Metal},
    math::quaternion::Quaternion,
    renderer::{OutputBuffers, Renderer},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Initialization");
    // Image
    let width = 1920;
    let height = 1080;

    let materials: Vec<MaterialDescriptor> = vec![
        MaterialDescriptor {
            label: Some("Bubble like".to_string()),
            material: Box::new(Dielectric {
                texture: Box::new(texture::Uniform(Rgb([0.7, 0.3, 0.3]))),
                ior: 0.8,
                invert_normal: false,
            }),
        },
        MaterialDescriptor {
            label: Some("Diffuse orange".to_string()),
            material: Box::new(Diffuse {
                texture: Box::new(texture::Checker {
                    odd: Box::new(texture::Uniform(Rgb([0.8, 0.6, 0.2]))),
                    even: Box::new(texture::Uniform(Rgb([0., 0., 0.]))),
                }),
            }),
        },
        MaterialDescriptor {
            label: Some("Gray metal".to_string()),
            material: Box::new(Metal {
                texture: Box::new(texture::Uniform(Rgb([0.8, 0.8, 0.8]))),
                roughness: 0.6,
            }),
        },
        MaterialDescriptor {
            label: Some("Ground".to_string()),
            material: Box::new(Diffuse {
                texture: Box::new(texture::Uniform(Rgb([0.2, 0.9, 0.3]))),
            }),
        },
        MaterialDescriptor {
            label: Some("Light".to_string()),
            material: Box::new(Emit {
                texture: Box::new(texture::Uniform(Rgb([2.5, 3.7, 3.9]))),
            }),
        },
        MaterialDescriptor {
            label: Some("Sky".to_string()),
            material: Box::new(Emit {
                texture: Box::new(texture::Uniform(Rgb([0.3, 0.3, 0.4]))),
            }),
        },
    ];

    let scene = HittableList(vec![
        Box::new(Sphere {
            label: Some("Bubble Sphere".to_string()),
            center: Vec3::new(0.0, 0.0, -1.),
            radius: 0.5,
            material: MaterialId(0),
        }),
        Box::new(Sphere {
            label: Some("Diffuse Sphere".to_string()),
            center: Vec3::new(1.0, 0.0, -1.),
            radius: 0.5,
            material: MaterialId(1),
        }),
        Box::new(Sphere {
            label: Some("Metal Sphere".to_string()),
            center: Vec3::new(-1.0, 0.0, -1.),
            radius: 0.5,
            material: MaterialId(2),
        }),
        Box::new(Sphere {
            label: Some("Ground".to_string()),
            center: Vec3::new(0.0, -100.5, -1.),
            radius: 100.,
            material: MaterialId(3),
        }),
        Box::new(Sphere {
            label: Some("Light".to_string()),
            center: Vec3::new(0.5, -0.4, -0.5),
            radius: 0.1,
            material: MaterialId(4),
        }),
    ]);

    let look_at = Vec3([0.0, 0.0, -1.0]);
    let look_from = Vec3([0.0, 0.0, 0.0]);
    let look_direction = look_at - look_from;
    let camera = Camera::new(
        width,
        height,
        f64::to_radians(90.),
        look_direction.length(),
        look_from,
        Quaternion::from_direction(&look_direction, &-Vec3::Z),
        0.0,
    );
    let renderer = Renderer {
        camera,
        scene,
        materials,
        options: renderer::RendererOptions {
            samples_per_pixel: 8,
            diffuse_depth: 20,
            gamma: 2.2,
            world_material: MaterialId(5),
        },
    };

    let mut output_buffers = OutputBuffers {
        normal: ImageBuffer::new(width, height),
        color: ImageBuffer::new(width, height),
        albedo: ImageBuffer::new(width, height),
        depth: ImageBuffer::new(width, height),
    };

    renderer.run_scene(&mut output_buffers);

    std::fs::create_dir_all("output/ldr")?;
    std::fs::create_dir_all("output/hdr")?;

    let depth = ConvertBuffer::<ImageBuffer<Rgb<f32>, Vec<f32>>>::convert(&output_buffers.depth);
    log::info!("Saving HDR images...");
    output_buffers.color.save("output/hdr/color.exr")?;
    output_buffers.normal.save("output/hdr/normal.exr")?;
    output_buffers.albedo.save("output/hdr/albedo.exr")?;
    depth.save("output/hdr/depth.exr")?;

    log::info!("Saving LDR images...");
    ConvertBuffer::<ImageBuffer<Rgb<u8>, Vec<u8>>>::convert(&output_buffers.color)
        .save("output/ldr/color.jpg")?;
    ConvertBuffer::<ImageBuffer<Rgb<u8>, Vec<u8>>>::convert(&output_buffers.normal)
        .save("output/ldr/normal.jpg")?;
    ConvertBuffer::<ImageBuffer<Rgb<u8>, Vec<u8>>>::convert(&output_buffers.albedo)
        .save("output/ldr/albedo.jpg")?;
    ConvertBuffer::<ImageBuffer<Rgb<u8>, Vec<u8>>>::convert(&depth).save("output/ldr/depth.jpg")?;
    Ok(())
}
