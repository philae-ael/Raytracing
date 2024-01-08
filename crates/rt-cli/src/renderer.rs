use std::{
    io::Write,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
};

use crate::{Dimensions, Spp};

use super::progress;

use image::{ImageBuffer, Rgb32FImage};
use rand::{distributions, seq::SliceRandom};
use rand::{prelude::Distribution, thread_rng};
use rayon::prelude::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use rt::{
    camera::{Camera, PixelCoord, ViewportCoord},
    integrators::Integrator,
    math::{point::Point, quaternion::LookAt, vec::Vec3},
    renderer::{Channel, GenericRenderResult, PixelRenderResult, RaySeries, World},
    utils::counter::counter,
};

use itertools::Itertools;
use rt::scene::Scene;

use anyhow::Result;

enum Message {
    Tile(Arc<TileMsg>),
    Stop,
}
pub struct TileMsg {
    pub tile_x: u32,
    pub tile_y: u32,
    pub data: Vec<PixelRenderResult>,
}

impl TileMsg {
    fn tile_bounds(&self, tile_size: u32, height: u32, width: u32) -> (u32, u32, u32, u32) {
        let x = self.tile_x * tile_size;
        let y = self.tile_y * tile_size;
        let tile_width = (x + tile_size).min(width) - x;
        let tile_height = (y + tile_size).min(height) - y;
        (x, y, tile_width, tile_height)
    }
}

pub struct RendererCreateInfo {
    pub dimension: Dimensions,
    pub spp: Spp,
    pub tile_size: u32,
    pub scene: Scene,
    pub shuffle_tiles: bool,
    pub integrator: Box<dyn Integrator>,
    pub allowed_error: Option<f32>,
}

pub struct Renderer {
    pub dimension: Dimensions,
    pub tile_size: u32,
    pub shuffle_tiles: bool,

    pub samples_per_pixel: Spp,
    pub allowed_error: Option<f32>,

    pub world: World,
    // TODO: make a pool of materials
    pub integrator: Box<dyn Integrator>,
    camera: Camera,
}

type Luma32FImage = image::ImageBuffer<image::Luma<f32>, Vec<f32>>;
pub type OutputBuffers = GenericRenderResult<Rgb32FImage, Luma32FImage>;

impl Renderer {
    pub fn new(tile_create_info: RendererCreateInfo) -> Self {
        let look_at = Point::new(0.0, 0.0, -1.0);
        let look_from = Point::ORIGIN;
        let look_direction = look_at - look_from;
        let camera = Camera::new(
            tile_create_info.dimension.width,
            tile_create_info.dimension.height,
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

        Self {
            dimension: tile_create_info.dimension,
            samples_per_pixel: tile_create_info.spp,
            tile_size: tile_create_info.tile_size,
            shuffle_tiles: tile_create_info.shuffle_tiles,
            allowed_error: tile_create_info.allowed_error,
            integrator: tile_create_info.integrator,
            world: World::from_scene(tile_create_info.scene),
            camera,
        }
    }

    pub fn run<F: FnMut(Arc<TileMsg>) + Send>(
        self,
        on_tile_rendered: F,
    ) -> anyhow::Result<OutputBuffers> {
        let width = self.dimension.width;
        let height = self.dimension.height;
        let tile_size = self.tile_size;

        let mut output_buffers = OutputBuffers {
            color: ImageBuffer::new(width, height),
            normal: ImageBuffer::new(width, height),
            position: ImageBuffer::new(width, height),
            albedo: ImageBuffer::new(width, height),
            z: ImageBuffer::new(width, height),
            ray_depth: ImageBuffer::new(width, height),
        };

        let mut push_tile_on_output_buffers = |msg: &TileMsg| {
            let (x, y, width, height) =
                msg.tile_bounds(self.tile_size, self.dimension.height, self.dimension.width);

            for i in 0..width {
                for j in 0..height {
                    let data_index = (i + width * j) as usize;
                    for channel in msg.data[data_index] {
                        match channel {
                            Channel::Color(c) => {
                                *output_buffers.color.get_pixel_mut(x + i, y + j) =
                                    c.to_srgb().into()
                            }
                            Channel::Normal(c) => {
                                *output_buffers.normal.get_pixel_mut(x + i, y + j) =
                                    c.to_srgb().into()
                            }
                            Channel::Albedo(c) => {
                                *output_buffers.albedo.get_pixel_mut(x + i, y + j) =
                                    c.to_srgb().into()
                            }
                            Channel::Position(c) => {
                                *output_buffers.position.get_pixel_mut(x + i, y + j) =
                                    c.to_srgb().into()
                            }
                            Channel::Z(c) => {
                                *output_buffers.z.get_pixel_mut(x + i, y + j) = c.into()
                            }
                            Channel::RayDepth(c) => {
                                *output_buffers.ray_depth.get_pixel_mut(x + i, y + j) = c.into()
                            }
                        }
                    }
                }
            }
        };

        let tile_count_x = (width as f32 / tile_size as f32).ceil() as u32;
        let tile_count_y = (height as f32 / tile_size as f32).ceil() as u32;

        let progress = match self.samples_per_pixel {
            Spp::Spp(s) => progress::Progress::new((s * tile_count_x * tile_count_y) as usize),
            Spp::Inf => progress::Progress::new_inf(),
        };

        let mut tiles_data = Vec::new();
        tiles_data.resize_with((tile_count_x * tile_count_y) as usize, || None);

        let generation_result: Result<()> = rayon::scope(|s| {
            let (tx, rx) = channel();

            log::info!("Generating image...");
            s.spawn(|_| {
                let mut on_tile_rendered = on_tile_rendered;
                let rx: Receiver<Message> = rx; // Force move without moving anything else
                let mut last_progress_update = std::time::Instant::now();
                for msg in rx.iter() {
                    match msg {
                        Message::Tile(tile_msg) => {
                            push_tile_on_output_buffers(&tile_msg);
                            on_tile_rendered(tile_msg);
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

            let mut tiles_indices = (0..tile_count_x)
                .cartesian_product(0..tile_count_y)
                .collect::<Vec<_>>();

            // reorder them if needed
            if self.shuffle_tiles {
                tiles_indices.shuffle(&mut thread_rng());
            }

            let process_sample_batch =
                |(tx, sample_count): &mut (Sender<Message>, u32),
                 ((tile_x, tile_y), data): ((u32, u32), &mut Option<Vec<RaySeries>>)|
                 -> Result<()> {
                    self.tile_worker((tile_x, tile_y), data, *sample_count);
                    progress.add(*sample_count as usize);

                    // Broadcast results to the thread which is in charge
                    tx.send(Message::Tile(Arc::new(TileMsg {
                        tile_x,
                        tile_y,
                        data: data
                            .as_ref()
                            .unwrap()
                            .iter()
                            .map(|x| x.as_pixelresult())
                            .collect::<Vec<_>>(),
                    })))?;

                    Ok(())
                };

            let samples_per_batch = 32;
            // Dispatch work
            match self.samples_per_pixel {
                Spp::Spp(s) => {
                    let mut samples_to_do = s;
                    while samples_to_do > 0 {
                        // Samples for this iteration
                        let samples = u32::min(samples_per_batch, samples_to_do);
                        samples_to_do -= samples;

                        tiles_indices
                            .par_iter()
                            .copied()
                            .zip(tiles_data.par_iter_mut())
                            .try_for_each_with((tx.clone(), samples), process_sample_batch)?;
                    }
                }
                Spp::Inf => {
                    for _sample_batch in 0.. {
                        tiles_indices
                            .par_iter()
                            .copied()
                            .zip(tiles_data.par_iter_mut())
                            .try_for_each_with(
                                (tx.clone(), samples_per_batch),
                                process_sample_batch,
                            )?;
                    }
                }
            };

            tx.send(Message::Stop)?;
            Ok(())
        });

        match generation_result {
            Ok(_) => log::info!("Image fully generated"),
            Err(err) => log::info!("Image generation interrupted: {}", err),
        };

        Ok(output_buffers)
    }

    fn tile_worker(
        &self,
        (tile_x, tile_y): (u32, u32),
        data: &mut Option<Vec<RaySeries>>,
        sample_count: u32,
    ) {
        let x_range =
            (tile_x * self.tile_size)..((tile_x + 1) * self.tile_size).min(self.dimension.width);
        let y_range =
            (tile_y * self.tile_size)..((tile_y + 1) * self.tile_size).min(self.dimension.height);
        let tile_width = x_range.len();
        let tile_height = y_range.len();

        let tile_data = if let Some(ref mut tile) = data {
            tile
        } else {
            let mut tile_data = Vec::new();
            tile_data.resize_with(tile_width * tile_height, RaySeries::default);
            *data = Some(tile_data);
            data.as_mut().unwrap()
        };

        for (j, y) in y_range.enumerate() {
            for (i, x) in x_range.clone().enumerate() {
                let index = j * tile_width + i;
                tile_data[index] = RaySeries::merge(
                    std::mem::take(&mut tile_data[index]),
                    self.pixel_worker(PixelCoord { x, y }, sample_count),
                );
            }
        }

        log::debug!("Tile {tile_x} {tile_y} done !");
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
