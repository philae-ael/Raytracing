mod file_output;
mod tev_streaming;

use anyhow::Result;
pub use file_output::FileOutput;
pub use tev_streaming::TevStreaming;

use crate::executor::{OutputBuffers, TileMsg};

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
