use std::collections::HashSet;

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

pub struct Renderer {
    pub streaming_outputs: Vec<Box<dyn StreamingOutput>>,
    pub final_outputs: Vec<Box<dyn FinalOutput>>,
    pub executor: Executor,
    pub multithreaded: bool,
}

impl Renderer {
    pub fn from_args(args: Args) -> Result<Self> {
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
            }
        };

        let mut this = Self {
            streaming_outputs: Vec::new(),
            final_outputs: Vec::new(),
            executor,
            multithreaded: !args.disable_threading,
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
        let output_buffers = timed_scope_log("Run tile renderer", || {
            let f = |msg| {
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
            };
            if self.multithreaded {
                self.executor.run_multithreaded(world, f)
            } else {
                self.executor.run_monothreaded(world, f)
            }
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
