use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use raytracing::utils::{counter, timer::timed_scope_log};

use crate::{
    output::{FileOutput, TevStreaming},
    tile_renderer::{OutputBuffers, TileMsg, TileRenderer, TileRendererCreateInfo},
    Args, AvailableOutput,
};

pub trait StreamingOutput: Send {
    fn send_msg(&mut self, msg: Arc<TileMsg>) -> Result<()>;
}
pub trait FinalOutput: Send {
    fn commit(&self, output_buffers: &OutputBuffers) -> Result<()>;
}

pub struct Cli {
    pub streaming_outputs: Vec<Box<dyn StreamingOutput>>,
    pub final_outputs: Vec<Box<dyn FinalOutput>>,
    pub tile_renderer: TileRenderer,
}
impl Cli {
    pub fn new(args: Args) -> Result<Self> {
        if args.no_threads {
            // Only one thread == Not Threaded
            rayon::ThreadPoolBuilder::new()
                .num_threads(1)
                .build_global()
                .unwrap();
        }

        let outputs: HashSet<AvailableOutput> = HashSet::from_iter(args.output.into_iter());
        let tile_size = 32;

        let mut this = Self {
            streaming_outputs: Vec::new(),
            final_outputs: Vec::new(),
            tile_renderer: TileRenderer::new(TileRendererCreateInfo {
                dimension: args.dimensions.clone(),
                spp: args.sample_per_pixel,
                tile_size,
                scene: args.scene.into(),
                shuffle_tiles: false,
                integrator: args.integrator.into(),
                allowed_error: args.allowed_error,
            }),
        };

        if outputs.contains(&AvailableOutput::Tev) {
            this.streaming_outputs.push(Box::new(TevStreaming::new(
                args.dimensions,
                tile_size,
                args.tev_path,
                args.tev_hostname,
            )?));
        }

        if outputs.contains(&AvailableOutput::File) {
            this.final_outputs.push(Box::new(FileOutput::new()));
        }

        Ok(this)
    }

    pub fn run(mut self) -> Result<()> {
        let output_buffers = timed_scope_log("Run tile renderer", || {
            self.tile_renderer.run(|msg| {
                let mut outputs = Vec::new();
                // Move tev_cli out of self, work with it and move it back in self
                std::mem::swap(&mut self.streaming_outputs, &mut outputs);

                for mut output in outputs.drain(..) {
                    match output.send_msg(msg.clone()) {
                        Ok(_) => self.streaming_outputs.push(output),
                        Err(err) => {
                            log::error!("{err}");
                        }
                    }
                }
            })
        })
        .res?;

        for final_output in self.final_outputs {
            final_output.commit(&output_buffers)?;
        }

        log::info!("Done");
        counter::report_counters();
        Ok(())
    }
}
