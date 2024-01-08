use anyhow::Result;
use image::{buffer::ConvertBuffer, ImageBuffer, Rgb};
use std::path::PathBuf;

use crate::{cli::Cli, tile_renderer::OutputBuffers};

pub struct FileOutput {
    pub hdr_outdir: Option<PathBuf>,
    pub ldr_outdir: Option<PathBuf>,
}

impl FileOutput {
    pub fn new(_cli: &Cli) -> Self {
        Self {
            hdr_outdir: Some("output/hdr/".into()),
            ldr_outdir: Some("output/ldr/".into()),
        }
    }

    pub fn commit(&self, output_buffers: OutputBuffers) -> Result<()> {
        if let Some(ref hdr_output) = self.hdr_outdir {
            let hdr_path = hdr_output.as_path();
            std::fs::create_dir_all(hdr_output)?;

            log::info!("Saving HDR images...");
            output_buffers.color.save(hdr_path.join("color.exr"))?;
            output_buffers.normal.save(hdr_path.join("normal.exr"))?;
            output_buffers.albedo.save(hdr_path.join("albedo.exr"))?;
            ConvertBuffer::<ImageBuffer<Rgb<f32>, Vec<f32>>>::convert(&output_buffers.depth)
                .save(hdr_path.join("depth.exr"))?;
            ConvertBuffer::<ImageBuffer<Rgb<f32>, Vec<f32>>>::convert(&output_buffers.ray_depth)
                .save(hdr_path.join("ray_depth.exr"))?;
        }
        if let Some(ref ldr_output) = self.ldr_outdir {
            let ldr_path = ldr_output.as_path();
            std::fs::create_dir_all(ldr_output)?;

            log::info!("Saving LDR images...");
            ConvertBuffer::<ImageBuffer<Rgb<u8>, Vec<u8>>>::convert(&output_buffers.color)
                .save(ldr_path.join("color.jpg"))?;
            ConvertBuffer::<ImageBuffer<Rgb<u8>, Vec<u8>>>::convert(&output_buffers.normal)
                .save(ldr_path.join("normal.jpg"))?;
            ConvertBuffer::<ImageBuffer<Rgb<u8>, Vec<u8>>>::convert(&output_buffers.albedo)
                .save(ldr_path.join("albedo.jpg"))?;
            ConvertBuffer::<ImageBuffer<Rgb<u8>, Vec<u8>>>::convert(&output_buffers.depth)
                .save(ldr_path.join("depth.jpg"))?;
            ConvertBuffer::<ImageBuffer<Rgb<u8>, Vec<u8>>>::convert(&output_buffers.ray_depth)
                .save(ldr_path.join("ray_depth.jpg"))?;
        }
        Ok(())
    }
}
