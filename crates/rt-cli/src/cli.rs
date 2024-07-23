use std::collections::HashSet;

use anyhow::Result;
use rt::utils::{counter, timer::timed_scope_log};

use crate::{
    output::{FileOutput, TevStreaming},
    executor::{OutputBuffers, Executor, ExecutorBuilder, TileMsg},
    Args, AvailableOutput,
};

pub trait StreamingOutput: Send {
    fn send_msg(&mut self, msg: &TileMsg) -> Result<()>;
}

struct DummyOutput {}
impl StreamingOutput for DummyOutput {
    fn send_msg(&mut self, _msg: &TileMsg) -> Result<()> {
        Ok(())
    }
}

pub trait FinalOutput: Send {
    fn commit(&self, output_buffers: &OutputBuffers) -> Result<()>;
}

pub struct Cli {
    pub streaming_outputs: Vec<Box<dyn StreamingOutput>>,
    pub final_outputs: Vec<Box<dyn FinalOutput>>,
    pub renderer: Executor,
}
impl Cli {
    pub fn new(args: Args) -> Result<Self> {
        if args.no_threads {
            log::warn!("Working on only one thread");
            // Only one thread == Not Threaded
            rayon::ThreadPoolBuilder::new()
                .num_threads(1)
                .build_global()
                .unwrap();
        }

        let outputs: HashSet<AvailableOutput> = HashSet::from_iter(args.output);
        let tile_size = 32;

        let renderer = {
            let mut renderer = ExecutorBuilder::default()
                .dimensions(args.dimensions)
                .spp(args.sample_per_pixel)
                .allowed_error(args.allowed_error);
            if let Some(tile_size) = args.tile_size {
                renderer = renderer.tile_size(tile_size);
            }
            renderer.build(args.integrator.into(), args.scene.into())
        };

        let mut this = Self {
            streaming_outputs: Vec::new(),
            final_outputs: Vec::new(),
            renderer,
        };

        for o in outputs {
            match o {
                AvailableOutput::Tev => {
                    this.streaming_outputs.push(Box::new(TevStreaming::new(
                        args.dimensions,
                        tile_size,
                        args.tev_path.clone(),
                        args.tev_hostname.clone(),
                    )?));
                }
                AvailableOutput::File => {
                    this.final_outputs.push(Box::new(FileOutput::new()));
                }
            }
        }

        Ok(this)
    }

    pub fn run(mut self) -> Result<()> {
        let output_buffers = timed_scope_log("Run tile renderer", || {
            self.renderer.run(|msg| {
                self.streaming_outputs
                    .iter_mut()
                    .for_each(|output| match output.send_msg(&msg) {
                        Ok(_) => (),
                        Err(err) => {
                            log::error!(
                                "Streaming output errored, it will not be used anymore: {err}"
                            );
                            *output = Box::new(DummyOutput {});
                        }
                    });
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
