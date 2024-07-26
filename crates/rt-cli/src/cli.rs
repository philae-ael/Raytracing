use std::{collections::HashSet, f32::consts::PI};

use anyhow::Result;
use rt::{
    aggregate::embree::EmbreeScene,
    camera::Camera,
    color::Rgb,
    loader::ObjLoaderExt,
    material::{texture::Uniform, Diffuse, Emit, MaterialDescriptor},
    math::{
        point::Point,
        quaternion::{LookAt, Quat},
        transform::Transform,
        vec::Vec3,
    },
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

pub struct Cli {
    pub streaming_outputs: Vec<Box<dyn StreamingOutput>>,
    pub final_outputs: Vec<Box<dyn FinalOutput>>,
    pub renderer: Executor,
    pub multithreaded: bool,
}
impl Cli {
    pub fn new(args: Args) -> Result<Self> {
        let outputs: HashSet<AvailableOutput> = HashSet::from_iter(args.output);
        let tile_size = 32;

        let executor = {
            let integrator = args.integrator.into();

            let mut scene = EmbreeScene::new(embree4_rs::Device::try_new(None).unwrap());
            scene.load_obj(
                "obj/dragon.obj",
                Transform {
                    translation: Vec3::new(0.0, 0.0, -1.0),
                    scale: 0.01 * Vec3::ONE,
                    rot: Quat::from_axis_angle(Vec3::Y, 1.1 * PI),
                },
                rt::material::MaterialId(0),
            );
            let scene = scene.commit();

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
                world: World {
                    objects: Box::new(scene),
                    lights: vec![Point::new(10.2, 80.0, 75.0)],
                    world_material: rt::material::MaterialId(1),
                    materials: vec![
                        MaterialDescriptor {
                            label: Some("Material".into()),
                            material: Box::new(Diffuse {
                                texture: Box::new(Uniform(Rgb::from_array([0.5; 3]))),
                            }),
                        },
                        MaterialDescriptor {
                            label: Some("Sky".into()),
                            material: Box::new(Emit {
                                texture: Box::new(Uniform(Rgb::from_array([0.2, 0.2, 0.2]))),
                            }),
                        },
                    ],
                },
                camera,
            }
        };

        let mut this = Self {
            streaming_outputs: Vec::new(),
            final_outputs: Vec::new(),
            renderer: executor,
            multithreaded: !args.disable_threading,
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
                self.renderer.run_multithreaded(f)
            } else {
                self.renderer.run_monothreaded(f)
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
