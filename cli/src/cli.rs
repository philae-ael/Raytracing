use std::collections::HashSet;

use anyhow::Result;

use crate::{
    output::{FileOutput, TevStreaming},
    tile_renderer::{TileMsg, TileRenderer, TileRendererCreateInfo},
    Args, AvailableOutput,
};

pub trait OutputStreaming: Send {
    fn send_msg(&mut self, msg: &TileMsg) -> Result<()>;
}

pub struct Cli {
    pub outputs: Vec<Box<dyn OutputStreaming>>,
    pub tile_renderer: TileRenderer,
}

impl Cli {
    pub fn new(args: Args) -> Result<Self> {
        let outputs: HashSet<AvailableOutput> = HashSet::from_iter(args.output.into_iter());
        let tile_size = 20;

        let mut this = Self {
            outputs: Vec::new(),
            tile_renderer: TileRenderer::new(TileRendererCreateInfo {
                dimension: args.dimensions.clone(),
                spp: args.sample_per_pixel,
                tile_size,
                scene: args.scene.into(),
                shuffle_tiles: false,
            }),
        };

        if outputs.contains(&AvailableOutput::Tev) {
            this.outputs.push(Box::new(TevStreaming::new(
                args.dimensions,
                tile_size,
                args.tev_path,
                args.tev_hostname,
            )?));
        }

        Ok(this)
    }

    pub fn run(mut self) -> Result<()> {
        let file_output = FileOutput::new(&self);

        let output_buffers = self.tile_renderer.run(|msg| {
            let mut outputs = Vec::new();
            // Move tev_cli out of self, work with it and move it back in self
            std::mem::swap(&mut self.outputs, &mut outputs);

            for mut output in outputs.drain(..) {
                match output.send_msg(&msg) {
                    Ok(_) => self.outputs.push(output),
                    Err(err) => {
                        log::error!("{err}");
                    }
                }
            }
        })?;

        file_output.commit(output_buffers)?;

        log::info!("Done");
        Ok(())
    }
}
