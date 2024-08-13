use anyhow::Result;
use itertools::Itertools;
use rt::utils::timer::timed_scope_log;
use rt::{renderer::World, utils::counter};

use crate::output::{OutputBuffers, OutputBuffersExt};
use crate::{
    executor::{Executor, TileMsg},
    output::{FileOutput, FinalOutput, StreamingOutput, TevStreaming},
    utils::{ExecutionMode, FromArgs, RenderRange},
    Args, AvailableOutput,
};

pub struct Renderer {
    pub streaming_outputs: Vec<Box<dyn StreamingOutput>>,
    pub final_outputs: Vec<Box<dyn FinalOutput>>,
    pub executor: Executor,
    pub execution_mode: ExecutionMode,
    pub pixel_range: RenderRange,
    pub sample_range: crate::utils::Spp,
}

impl FromArgs for Renderer {
    fn from_args(args: &Args) -> Self {
        log::info!("building renderer");
        let mut streaming_outputs = Vec::<Box<dyn StreamingOutput>>::new();
        let mut final_outputs = Vec::<Box<dyn FinalOutput>>::new();

        for o in args.output.iter().copied().unique() {
            match o {
                AvailableOutput::Tev => {
                    streaming_outputs.push(Box::new(
                        TevStreaming::new(
                            args.dimensions,
                            args.tev_path.clone(),
                            args.tev_hostname.clone(),
                        )
                        .expect("can't create tev output"),
                    ));
                }
                AvailableOutput::File => {
                    final_outputs.push(Box::new(FileOutput::new()));
                }
            }
        }

        Renderer {
            streaming_outputs,
            final_outputs,
            executor: FromArgs::from_args(args),
            execution_mode: args.execution_mode,
            sample_range: FromArgs::from_args(args),
            pixel_range: FromArgs::from_args(args),
        }
    }
}

impl Renderer {
    pub fn run(mut self, world: &World) -> Result<()> {
        log::info!("rendering");
        let mut output_buffers = OutputBuffers {
            channels: Vec::new(),
        };

        timed_scope_log("run tile renderer", || {
            let dim = self.executor.dimension;
            let f = |msg: &TileMsg| {
                for (index, (x, y)) in msg.tile.into_iter().enumerate() {
                    output_buffers.convert(&msg.data[index], x, y, dim);
                }
                self.streaming_outputs
                    .iter_mut()
                    .for_each(|output| output.send_msg(msg).unwrap());
            };

            match self.execution_mode {
                ExecutionMode::Multithreaded => {
                    log::info!("execution mode: multithreaded");
                    self.executor
                        .run_multithreaded(world, f, self.pixel_range, self.sample_range)
                }
                ExecutionMode::Monothreaded => {
                    log::info!("execution mode: monothreaded");
                    self.executor
                        .run_monothreaded(world, f, self.pixel_range, self.sample_range)
                }
            }
        })
        .res?;

        for final_output in self.final_outputs {
            final_output.commit(&output_buffers)?;
        }

        counter::report_counters();
        Ok(())
    }
}
