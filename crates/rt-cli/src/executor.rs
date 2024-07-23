use std::{
    io::Write,
    sync::mpsc::{channel, Receiver},
};

use crate::{
    tile::{Tile, Tiler},
    Dimensions, Spp,
};

use super::progress;

use image::{ImageBuffer, Rgb32FImage};
use rand::distributions;
use rand::prelude::Distribution;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use rt::{
    camera::{Camera, PixelCoord, ViewportCoord},
    color::{Luma, Rgb},
    integrators::Integrator,
    math::{point::Point, quaternion::LookAt, vec::Vec3},
    renderer::{GenericRenderResult, PixelRenderResult, RaySeries, World},
    utils::counter::counter,
};

use rt::scene::Scene;

enum Message {
    Tile(TileMsg),
    Stop,
}

pub struct TileMsg {
    pub tile: Tile,
    pub data: Vec<PixelRenderResult>,
}

pub struct ExecutorBuilder {
    pub dimension: Dimensions,
    pub spp: Spp,
    pub tile_size: u32,
    pub allowed_error: Option<f32>,
}

impl Default for ExecutorBuilder {
    fn default() -> Self {
        Self {
            dimension: Dimensions {
                width: 800,
                height: 600,
            },
            spp: Spp::Spp(32),
            tile_size: 32,
            allowed_error: None,
        }
    }
}

impl ExecutorBuilder {
    pub fn dimensions(mut self, dim: Dimensions) -> Self {
        self.dimension = dim;
        self
    }
    pub fn spp(mut self, spp: Spp) -> Self {
        self.spp = spp;
        self
    }

    pub fn tile_size(mut self, tile_size: u32) -> Self {
        self.tile_size = tile_size;
        self
    }

    pub fn allowed_error(mut self, allowed_error: Option<f32>) -> Self {
        self.allowed_error = allowed_error;
        self
    }

    pub fn build(self, integrator: Box<dyn Integrator>, scene: Scene) -> Executor {
        let look_at = Point::new(0.0, 0.0, -1.0);
        let look_from = Point::ORIGIN;
        let look_direction = look_at - look_from;
        let camera = Camera::new(
            self.dimension.width,
            self.dimension.height,
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
            dimension: self.dimension,
            samples_per_pixel: self.spp,
            tile_size: self.tile_size,
            allowed_error: self.allowed_error,
            integrator,
            world: World::from_scene(scene),
            camera,
        }
    }
}

pub struct Executor {
    pub dimension: Dimensions,
    pub tile_size: u32,

    pub samples_per_pixel: Spp,
    pub allowed_error: Option<f32>,

    pub world: World,
    // TODO: make a pool of materials
    pub integrator: Box<dyn Integrator>,
    camera: Camera,
}

type Luma32FImage = image::ImageBuffer<image::Luma<f32>, Vec<f32>>;
pub type OutputBuffers = GenericRenderResult<Rgb32FImage, Luma32FImage>;

trait OutputBuffersExt<T, L> {
    fn convert(&mut self, d: GenericRenderResult<T, L>, x: u32, y: u32);
    fn new(d: Dimensions) -> Self;
}
impl OutputBuffersExt<Rgb, Luma> for OutputBuffers {
    fn convert(&mut self, d: PixelRenderResult, x: u32, y: u32) {
        *self.color.get_pixel_mut(x, y) = d.color.to_srgb().into();
        *self.normal.get_pixel_mut(x, y) = d.normal.to_srgb().into();
        *self.albedo.get_pixel_mut(x, y) = d.albedo.to_srgb().into();
        *self.position.get_pixel_mut(x, y) = d.position.to_srgb().into();
        *self.z.get_pixel_mut(x, y) = d.z.into();
        *self.ray_depth.get_pixel_mut(x, y) = d.ray_depth.into();
    }

    fn new(Dimensions { width, height }: Dimensions) -> Self {
        Self {
            color: ImageBuffer::new(width, height),
            normal: ImageBuffer::new(width, height),
            position: ImageBuffer::new(width, height),
            albedo: ImageBuffer::new(width, height),
            z: ImageBuffer::new(width, height),
            ray_depth: ImageBuffer::new(width, height),
        }
    }
}

impl Executor {
    pub fn run<F: FnMut(TileMsg) + Send>(
        self,
        on_tile_rendered: F,
    ) -> anyhow::Result<OutputBuffers> {
        let mut output_buffers = OutputBuffers::new(self.dimension);

        let tiler = Tiler {
            width: self.dimension.width,
            height: self.dimension.height,
            x_grainsize: self.tile_size,
            y_grainsize: self.tile_size,
        };

        let progress = match self.samples_per_pixel {
            Spp::Spp(s) => progress::Progress::new(s as usize * tiler.tile_count()),
            Spp::Inf => progress::Progress::new_inf(),
        };

        let mut tiles_data: Vec<Vec<RaySeries>> = tiler
            .into_iter()
            .map(|tile| {
                let mut c = Vec::new();
                c.resize_with(tile.width() * tile.height(), Default::default);
                c
            })
            .collect();

        let generation_result = rayon::scope(|s| {
            let (tx, rx) = channel();

            log::info!("Generating image...");
            s.spawn(|_| {
                let mut on_tile_rendered = on_tile_rendered;
                let rx: Receiver<Message> = rx; // Force move without moving anything else
                let mut last_progress_update = std::time::Instant::now();
                for msg in rx.iter() {
                    match msg {
                        Message::Tile(msg) => {
                            for (index, (x, y)) in msg.tile.into_iter().enumerate() {
                                output_buffers.convert(msg.data[index], x, y);
                            }
                            on_tile_rendered(msg);
                        }
                        Message::Stop => {
                            break;
                        }
                    }

                    if last_progress_update.elapsed() >= std::time::Duration::from_millis(300) {
                        print!("\r{progress}");
                        let _ = std::io::stdout().flush();
                        last_progress_update = std::time::Instant::now();
                    }
                }
                println!("\r{progress}");
            });

            let mut dispatch = |sample_count| {
                tiler
                    .into_par_iter()
                    .zip(tiles_data.par_iter_mut())
                    .map(|(tile, data)| {
                        self.tile_worker(tile, data, sample_count);
                        progress.add(sample_count as usize);

                        TileMsg {
                            tile,
                            data: data.iter().map(|x| x.as_pixelresult()).collect::<Vec<_>>(),
                        }
                    })
                    .try_for_each_init(
                        || tx.clone(),
                        |tx, msg: TileMsg| tx.send(Message::Tile(msg)),
                    )
            };

            let sample_batch_size = 32;
            match self.samples_per_pixel {
                Spp::Spp(s) => {
                    let mut samples_to_do = s;
                    while samples_to_do > 0 {
                        // Samples for this iteration
                        let samples = u32::min(sample_batch_size, samples_to_do);
                        samples_to_do -= samples;

                        dispatch(samples)?;
                    }
                }
                Spp::Inf => {
                    for _sample_batch in 0.. {
                        dispatch(sample_batch_size)?;
                    }
                }
            };
            tx.send(Message::Stop)
        });

        match generation_result {
            Ok(_) => log::info!("Image fully generated"),
            Err(err) => log::info!("Image generation interrupted: {}", err),
        };

        Ok(output_buffers)
    }

    fn tile_worker(&self, tile: Tile, data: &mut [RaySeries], sample_count: u32) {
        log::trace!("working on tile {tile:?}");
        for (index, (x, y)) in tile.into_iter().enumerate() {
            data[index] = RaySeries::merge(
                std::mem::take(&mut data[index]),
                self.pixel_worker(PixelCoord { x, y }, sample_count),
            );
        }
    }

    fn pixel_worker(&self, coords: PixelCoord, samples: u32) -> RaySeries {
        let ViewportCoord { vx, vy } = ViewportCoord::from_pixel_coord(&self.camera, coords);
        let pixel_width = 1. / (self.camera.width as f32 - 1.);
        let pixel_height = 1. / (self.camera.height as f32 - 1.);
        let distribution_x = distributions::Uniform::new(-pixel_width / 2., pixel_width / 2.);
        let distribution_y = distributions::Uniform::new(-pixel_height / 2., pixel_height / 2.);

        let mut rng = rand::thread_rng();
        let mut ray_series = RaySeries::default();

        for _ in 0..samples {
            counter!("Samples");
            let dvx = distribution_x.sample(&mut rng);
            let dvy = distribution_y.sample(&mut rng);
            let camera_ray = self.camera.ray(vx + dvx, vy + dvy, &mut rng);

            ray_series.add_sample(self.integrator.ray_cast(&self.world, camera_ray, 0));

            if let Some(allowed_error) = self.allowed_error {
                if ray_series.color.is_precise_enough(allowed_error).is_some() {
                    counter!("Adaptative sampling break");
                    break;
                }
            }
        }

        ray_series
    }
}
