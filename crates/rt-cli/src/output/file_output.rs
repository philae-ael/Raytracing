use anyhow::Result;
use image::{buffer::ConvertBuffer, ImageBuffer, Rgb};
use std::path::PathBuf;

use crate::{cli::FinalOutput, executor::OutputBuffers};

pub struct FileOutput {
    pub hdr_outdir: Option<PathBuf>,
    pub ldr_outdir: Option<PathBuf>,
}

impl FileOutput {
    pub fn new() -> Self {
        Self {
            hdr_outdir: Some("output/hdr/".into()),
            ldr_outdir: Some("output/ldr/".into()),
        }
    }
}

impl FinalOutput for FileOutput {
    fn commit(&self, output_buffers: &OutputBuffers) -> Result<()> {
        if let Some(ref hdr_output) = self.hdr_outdir {
            let convert_luma = ConvertBuffer::<ImageBuffer<Rgb<f32>, Vec<f32>>>::convert;
            let hdr_path = hdr_output.as_path();
            std::fs::create_dir_all(hdr_output)?;

            log::info!("Saving HDR images...");
            for buff in output_buffers.as_ref().into_iter() {
                match buff {
                    rt::renderer::Channel::Color(color) => {
                        color.save(hdr_path.join("color.exr"))
                    }
                    rt::renderer::Channel::Position(position) => {
                        position.save(hdr_path.join("position.exr"))
                    }
                    rt::renderer::Channel::Normal(normal) => {
                        normal.save(hdr_path.join("normal.exr"))
                    }
                    rt::renderer::Channel::Albedo(albedo) => {
                        albedo.save(hdr_path.join("albedo.exr"))
                    }
                    rt::renderer::Channel::Z(z) => {
                        convert_luma(z).save(hdr_path.join("depth.exr"))
                    }
                    rt::renderer::Channel::RayDepth(ray_depth) => {
                        convert_luma(ray_depth).save(hdr_path.join("ray_depth.exr"))
                    }
                }?
            }
        }
        if let Some(ref ldr_output) = self.ldr_outdir {
            let convert_luma = ConvertBuffer::<ImageBuffer<Rgb<u8>, Vec<u8>>>::convert;
            let convert_rgb = ConvertBuffer::<ImageBuffer<Rgb<u8>, Vec<u8>>>::convert;
            let ldr_path = ldr_output.as_path();
            std::fs::create_dir_all(ldr_output)?;

            log::info!("Saving LDR images...");
            for buff in output_buffers.as_ref().into_iter() {
                match buff {
                    rt::renderer::Channel::Color(color) => {
                        convert_rgb(color).save(ldr_path.join("color.jpg"))
                    }
                    rt::renderer::Channel::Normal(normal) => {
                        convert_rgb(normal).save(ldr_path.join("normal.jpg"))
                    }
                    rt::renderer::Channel::Position(position) => {
                        convert_rgb(position).save(ldr_path.join("position.jpg"))
                    }
                    rt::renderer::Channel::Albedo(albedo) => {
                        convert_rgb(albedo).save(ldr_path.join("albedo.jpg"))
                    }
                    rt::renderer::Channel::Z(z) => {
                        convert_luma(z).save(ldr_path.join("depth.jpg"))
                    }
                    rt::renderer::Channel::RayDepth(ray_depth) => {
                        convert_luma(ray_depth).save(ldr_path.join("ray_depth.jpg"))
                    }
                }?
            }
        }
        Ok(())
    }
}
