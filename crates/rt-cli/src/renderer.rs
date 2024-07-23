use std::{
    io::Write,
    sync::mpsc::{channel, Receiver},
};

use crate::{Dimensions, Spp};

use super::progress;

use image::{ImageBuffer, Rgb32FImage};
use rand::distributions;
use rand::prelude::Distribution;
use rayon::prelude::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use rt::{
    camera::{Camera, PixelCoord, ViewportCoord},
    color::{Luma, Rgb},
    integrators::Integrator,
    math::{point::Point, quaternion::LookAt, vec::Vec3},
    renderer::{GenericRenderResult, PixelRenderResult, RaySeries, World},
    utils::counter::counter,
};

use itertools::Itertools;
use rt::scene::Scene;

enum Message {
    Tile(TileMsg),
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
    pub integrator: Box<dyn Integrator>,
    pub allowed_error: Option<f32>,
}

pub struct Renderer {
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
}

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
            allowed_error: tile_create_info.allowed_error,
            integrator: tile_create_info.integrator,
            world: World::from_scene(tile_create_info.scene),
            camera,
        }
    }

    pub fn run<F: FnMut(TileMsg) + Send>(
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

        let tile_count_x = (width as f32 / tile_size as f32).ceil() as u32;
        let tile_count_y = (height as f32 / tile_size as f32).ceil() as u32;

        let progress = match self.samples_per_pixel {
            Spp::Spp(s) => progress::Progress::new((s * tile_count_x * tile_count_y) as usize),
            Spp::Inf => progress::Progress::new_inf(),
        };

        let mut tiles_data: Vec<Vec<RaySeries>> = (0..tile_count_x)
            .cartesian_product(0..tile_count_y)
            .map(|(tile_x, tile_y)| {
                let x_range = (tile_x * self.tile_size)
                    ..((tile_x + 1) * self.tile_size).min(self.dimension.width);
                let y_range = (tile_y * self.tile_size)
                    ..((tile_y + 1) * self.tile_size).min(self.dimension.height);
                let tile_width = x_range.len();
                let tile_height = y_range.len();
                let mut c = Vec::new();
                c.resize_with(tile_width * tile_height, Default::default);
                c
            })
            .collect();

        let mut push_tile_on_output_buffers = |msg: &TileMsg| {
            let (x, y, width, height) =
                msg.tile_bounds(self.tile_size, self.dimension.height, self.dimension.width);

            for i in 0..width {
                for j in 0..height {
                    output_buffers.convert(msg.data[(i + width * j) as usize], x + i, y + j);
                }
            }
        };

        let generation_result = rayon::scope(|s| {
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

            let tiles_indices = (0..tile_count_x)
                .cartesian_product(0..tile_count_y)
                .collect::<Vec<_>>();

            let mut dispatch = |sample_count| {
                tiles_indices
                    .par_iter()
                    .copied()
                    .zip(tiles_data.par_iter_mut())
                    .map(|((tile_x, tile_y), data)| {
                        self.tile_worker((tile_x, tile_y), data, sample_count);
                        progress.add(sample_count as usize);

                        TileMsg {
                            tile_x,
                            tile_y,
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

    fn tile_worker(&self, (tile_x, tile_y): (u32, u32), data: &mut [RaySeries], sample_count: u32) {
        let x_range =
            (tile_x * self.tile_size)..((tile_x + 1) * self.tile_size).min(self.dimension.width);
        let y_range =
            (tile_y * self.tile_size)..((tile_y + 1) * self.tile_size).min(self.dimension.height);
        let tile_width = x_range.len();

        for (j, y) in y_range.enumerate() {
            for (i, x) in x_range.clone().enumerate() {
                let index = j * tile_width + i;
                data[index] = RaySeries::merge(
                    std::mem::take(&mut data[index]),
                    self.pixel_worker(PixelCoord { x, y }, sample_count),
                );
            }
        }

        log::trace!("Tile {tile_x} {tile_y} done !");
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
