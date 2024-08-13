use anyhow::Result;
use image::{buffer::ConvertBuffer, ImageBuffer, Rgb};
use std::path::PathBuf;

use super::{FinalOutput, OutputBuffers};

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
            for buff in &output_buffers.channels {
                match buff {
                    rt::renderer::Channel::RgbChannel(chan, c) => {
                        c.save(hdr_path.join(chan.to_string() + ".exr"))
                    }
                    rt::renderer::Channel::LumaChannel(chan, c) => {
                        convert_luma(c).save(hdr_path.join(chan.to_string() + ".exr"))
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
            for buff in &output_buffers.channels {
                match buff {
                    rt::renderer::Channel::RgbChannel(chan, c) => {
                        convert_rgb(c).save(ldr_path.join(chan.to_string() + ".jpeg"))
                    }
                    rt::renderer::Channel::LumaChannel(chan, c) => {
                        convert_luma(c).save(ldr_path.join(chan.to_string() + ".jpeg"))
                    }
                }?
            }
        }
        Ok(())
    }
}
