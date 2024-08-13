mod file_output;
mod tev_streaming;

use core::panic;

use anyhow::Result;
pub use file_output::FileOutput;
use image::{ImageBuffer, Rgb32FImage};
use rt::{
    color::{ColorspaceConversion, Luma, Rgb},
    renderer::{Channel, GenericRenderResult, PixelRenderResult},
};
pub use tev_streaming::TevStreaming;

use crate::{executor::TileMsg, utils::Dimensions};

type Luma32FImage = image::ImageBuffer<image::Luma<f32>, Vec<f32>>;
pub type OutputBuffers = GenericRenderResult<Rgb32FImage, Luma32FImage>;

pub trait OutputBuffersExt<T, L> {
    fn convert(&mut self, d: &GenericRenderResult<T, L>, x: u32, y: u32, dim: Dimensions);
}
impl OutputBuffersExt<Rgb, Luma> for OutputBuffers {
    fn convert(
        &mut self,
        d: &PixelRenderResult,
        x: u32,
        y: u32,
        Dimensions { width, height }: Dimensions,
    ) {
        if self.channels.len() == 0 {
            for chan in &d.channels {
                match chan {
                    Channel::RgbChannel(name, _) => self
                        .channels
                        .push(Channel::RgbChannel(*name, ImageBuffer::new(width, height))),
                    Channel::LumaChannel(name, _) => self
                        .channels
                        .push(Channel::LumaChannel(*name, ImageBuffer::new(width, height))),
                }
            }
        }

        assert_eq!(self.channels.len(), d.channels.len());

        for (chan1, chan2) in self.channels.iter_mut().zip(&d.channels) {
            match (chan1, chan2) {
                (Channel::RgbChannel(n1, c), Channel::RgbChannel(n2, d)) => {
                    assert_eq!(n1, n2);
                    *c.get_pixel_mut(x, y) = d.convert().into();
                }
                (Channel::LumaChannel(n1, c), Channel::LumaChannel(n2, d)) => {
                    assert_eq!(n1, n2);
                    *c.get_pixel_mut(x, y) = (*d).into();
                }
                _ => panic!("malformed pixel render result!"),
            }
        }
    }
}

pub trait StreamingOutput: Send {
    fn send_msg(&mut self, msg: &TileMsg) -> Result<()>;
}

pub struct DummyOutput {}
impl StreamingOutput for DummyOutput {
    fn send_msg(&mut self, _msg: &TileMsg) -> Result<()> {
        Ok(())
    }
}

pub trait FinalOutput: Send {
    fn commit(&self, output_buffers: &OutputBuffers) -> Result<()>;
}
