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
    renderer::{GenericRenderResult, PixelRenderResult, RaySeries, World},
    utils::counter::counter,
};

enum Message {
    Tile(TileMsg),
    Stop,
}

pub struct TileMsg {
    pub tile: Tile,
    pub data: Vec<PixelRenderResult>,
}

pub struct Executor {
    pub dimension: Dimensions,
    pub tile_size: u32,

    pub samples_per_pixel: Spp,
    pub allowed_error: Option<f32>,

    pub world: World,
    // TODO: make a pool of materials
    pub integrator: Box<dyn Integrator>,
    pub camera: Camera,
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
    pub fn run_multithreaded<F: FnMut(TileMsg) + Send>(
        self,
        mut on_tile_rendered: F,
    ) -> anyhow::Result<OutputBuffers> {
        log::debug!("Monothreaded");

        let mut output_buffers = OutputBuffers::new(self.dimension);
        let (tx, rx) = channel();
        let (mut ctx, progress) = self.build_ctx(|msg| {
            tx.send(Message::Tile(msg)).unwrap();
        });
        let generation_result = rayon::scope(|s| {
            log::info!("Generating image...");

            log::info!("Generating image...");
            s.spawn(|_| {
                let output_buffers = &mut output_buffers;
                let progress = &progress;
                let rx: Receiver<Message> = rx; // Force move without moving anything else
                let mut last_progress_update = std::time::Instant::now();

                loop {
                    let Some(msg) = rx.try_recv().ok() else {
                        rayon::yield_now();
                        continue;
                    };
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
                print!("\r{progress}\n");
                let _ = std::io::stdout().flush();
            });

            for samples in SampleCounter::new(32, ctx.executor.samples_per_pixel) {
                ctx.dispatch_async(samples, &progress);
            }
            tx.send(Message::Stop)
        });

        match generation_result {
            Ok(_) => {
                log::info!("Image fully generated")
            }
            Err(err) => log::info!("Image generation interrupted: {}", err),
        };
        Ok(output_buffers)
    }

    pub fn run_monothreaded<F: FnMut(TileMsg)>(
        self,
        on_tile_rendered: F,
    ) -> anyhow::Result<OutputBuffers> {
        log::debug!("Monothreaded");

        let mut output_buffers = OutputBuffers::new(self.dimension);
        let (mut ctx, progress) = self.build_ctx(on_tile_rendered);

        log::info!("Generating image...");

        for samples in SampleCounter::new(32, ctx.executor.samples_per_pixel) {
            ctx.dispatch_sync(samples, &mut output_buffers, &progress);
        }
        print!("\r{progress}\n");

        log::info!("Image fully generated");

        Ok(output_buffers)
    }

    fn build_ctx<F>(self, on_tile_rendered: F) -> (Ctx<F>, progress::Progress) {
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

        (
            Ctx {
                tiler,
                tiles_data: tiler
                    .into_iter()
                    .map(|tile| {
                        let mut c = Vec::new();
                        c.resize_with(tile.width() * tile.height(), Default::default);
                        c
                    })
                    .collect::<Vec<Vec<RaySeries>>>(),
                on_tile_rendered,
                executor: self,
            },
            progress,
        )
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

struct Ctx<F> {
    tiler: Tiler,
    tiles_data: Vec<Vec<RaySeries>>,
    on_tile_rendered: F,
    executor: Executor,
}

impl<F: FnMut(TileMsg)> Ctx<F> {
    fn dispatch_sync(
        &mut self,
        sample_count: u32,
        output_buffers: &mut OutputBuffers,
        progress: &progress::Progress,
    ) {
        for (tile, data) in self.tiler.into_iter().zip(self.tiles_data.iter_mut()) {
            self.executor.tile_worker(tile, data, sample_count);
            progress.add(sample_count as _);
            print!("\r{progress}");
            let _ = std::io::stdout().flush();

            let msg = TileMsg {
                tile,
                data: data.iter().map(|x| x.as_pixelresult()).collect::<Vec<_>>(),
            };

            for (index, (x, y)) in msg.tile.into_iter().enumerate() {
                output_buffers.convert(msg.data[index], x, y);
            }
            (self.on_tile_rendered)(msg);
        }
    }
}

impl<F: Fn(TileMsg) + Sync + Send> Ctx<F> {
    fn dispatch_async(&mut self, sample_count: u32, progress: &progress::Progress) {
        self.tiler
            .into_par_iter()
            .zip(self.tiles_data.par_iter_mut())
            .map(|(tile, data)| {
                self.executor.tile_worker(tile, data, sample_count);
                progress.add(sample_count as _);

                TileMsg {
                    tile,
                    data: data.iter().map(|x| x.as_pixelresult()).collect::<Vec<_>>(),
                }
            })
            .for_each(&self.on_tile_rendered)
    }
}

struct SampleCounter {
    batch_size: u32,
    spp: Spp,
}

impl SampleCounter {
    fn new(batch_size: u32, spp: Spp) -> Self {
        Self { batch_size, spp }
    }
}

impl Iterator for SampleCounter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.spp {
            Spp::Spp(s) => {
                if *s == 0 {
                    return None;
                }
                let samples = u32::min(self.batch_size, *s);
                *s -= samples;
                Some(samples)
            }
            Spp::Inf => Some(self.batch_size),
        }
    }
}
