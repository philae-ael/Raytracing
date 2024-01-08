use image::{buffer::ConvertBuffer, ImageBuffer, Rgb};

use renderer::renderer::{DefaultRenderer, OutputBuffers, Renderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Initialization");
    // Image
    let width = 1920;
    let height = 1080;

    let renderer: Renderer = DefaultRenderer { width, height }.into();
    let mut output_buffers = OutputBuffers {
        color: ImageBuffer::new(width, height),
        normal: ImageBuffer::new(width, height),
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
