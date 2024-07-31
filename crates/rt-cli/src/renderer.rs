use std::{collections::HashSet, ops::Range};

use anyhow::Result;
use clap::ValueEnum;
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

#[derive(Debug, Clone, ValueEnum, Copy)]
pub enum ExecutionMode {
    Multithreaded,
    Monothreaded,
}

#[derive(Debug, Clone)]
pub struct RenderRange {
    pub x: Range<u32>,
    pub y: Range<u32>,
}

impl std::str::FromStr for RenderRange {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let err = Err("expected monothreaded, multithreaded or a simple pixel `x`x`y`x`sample` eg 1x2x4 for the pixel 1 2 at sample 4");
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
        let None = it.next() else {
            return err;
        };

        Ok(RenderRange { x, y })
    }
}

pub struct Renderer {
    pub streaming_outputs: Vec<Box<dyn StreamingOutput>>,
    pub final_outputs: Vec<Box<dyn FinalOutput>>,
    pub executor: Executor,
    pub execution_mode: ExecutionMode,
    pub pixel_range: RenderRange,
    pub sample_range: crate::utils::Spp,
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
                tile_size: args.tile_size,
                allowed_error: args.allowed_error,
                spp: args.spp,
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
            sample_range: args
                .sample_range
                .unwrap_or(crate::utils::Spp::Spp(0..args.spp)),
            pixel_range: args.range.unwrap_or(RenderRange {
                x: 0..args.dimensions.width,
                y: 0..args.dimensions.height,
            }),
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
