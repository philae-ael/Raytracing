use std::{collections::HashSet, ops::Range};

use anyhow::Result;
use rt::{
    camera::Camera,
    math::{point::Point, quaternion::LookAt, vec::Vec3},
    renderer::World,
    utils::{counter, timer::timed_scope_log},
};

use crate::{
    executor::{Executor, OutputBuffers, TileMsg},
    output::{FileOutput, TevStreaming},
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

#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Multithreaded,
    Monothreaded,
    PixelRange {
        x: Range<u32>,
        y: Range<u32>,
        sample: Range<u32>,
    },
}
impl std::str::FromStr for ExecutionMode {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let err = Err("expected monothreaded, multithreaded or a simple pixel `x`x`y`x`sample` eg 1x2x4 for the pixel 1 2 at sample 4");
        match s.to_lowercase().as_str() {
            "multithreaded" => Ok(Self::Multithreaded),
            "monothreaded" => Ok(Self::Monothreaded),
            s => {
                let mut it = s.split('x').flat_map(|x| {
                    let Some(x) = x.split_once("..") else {
                        return x.parse::<u32>().ok().map(|x| x..(x + 1));
                    };

                    Some(x.0.parse().ok()?..x.1.parse().ok()?)
                });

                let Some(x) = it.next() else {
                    return err;
                };
                let Some(y) = it.next() else {
                    return err;
                };
                let Some(sample) = it.next() else {
                    return err;
                };
                let None = it.next() else {
                    return err;
                };

                Ok(ExecutionMode::PixelRange { x, y, sample })
            }
        }
    }
}

pub struct Renderer {
    pub streaming_outputs: Vec<Box<dyn StreamingOutput>>,
    pub final_outputs: Vec<Box<dyn FinalOutput>>,
    pub executor: Executor,
    pub execution_mode: ExecutionMode,
}

impl Renderer {
    pub fn from_args(args: Args) -> Result<Self> {
        log::info!("building renderer");
        let outputs: HashSet<AvailableOutput> = HashSet::from_iter(args.output);

        let executor = {
            let integrator = args.integrator.into();
            let look_at = Point::new(0.0, 0.0, -1.0);
            let look_from = Point::ORIGIN;
            let look_direction = look_at - look_from;
            let camera = Camera::new(
                args.dimensions.width,
                args.dimensions.height,
                f32::to_radians(70.),
                look_direction.length(),
                look_from,
                LookAt {
                    direction: look_direction,
                    forward: Vec3::NEG_Z,
                }
                .into(),
                0.0,
            );

            Executor {
                dimension: args.dimensions,
                samples_per_pixel: args.sample_per_pixel,
                tile_size: args.tile_size.unwrap_or(32),
                allowed_error: args.allowed_error,
                integrator,
                camera,
                seed: args.seed,
            }
        };

        let mut this = Self {
            streaming_outputs: Vec::new(),
            final_outputs: Vec::new(),
            executor,
            execution_mode: args.execution_mode,
        };

        for o in outputs {
            match o {
                AvailableOutput::Tev => {
                    this.streaming_outputs.push(Box::new(TevStreaming::new(
                        args.dimensions,
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

    pub fn run(mut self, world: &World) -> Result<()> {
        log::info!("rendering");
        let output_buffers = timed_scope_log("run tile renderer", || {
            let f = |msg| {
                self.streaming_outputs
                    .iter_mut()
                    .for_each(|output| match output.send_msg(&msg) {
                        Ok(_) => (),
                        Err(err) => {
                            log::error!(
                                "streaming output errored, it will not be used anymore: {err}"
                            );
                            *output = Box::new(DummyOutput {});
                        }
                    });
            };
            match self.execution_mode {
                ExecutionMode::Multithreaded => {
                    log::info!("execution mode: multithreaded");
                    self.executor.run_multithreaded(world, f)
                }
                ExecutionMode::Monothreaded => {
                    log::info!("execution mode: monothreaded");
                    self.executor.run_monothreaded(world, f)
                }
                ExecutionMode::PixelRange { x, y, sample } => {
                    self.executor.run_pixels(world, f, x, y, sample)
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
