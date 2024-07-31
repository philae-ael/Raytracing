use rt::{
    filter::{BoxFilter, Filter, TriangleFilter},
    math::vec::Vec2,
    sampler::{Sampler, StratifiedSampler},
    Seed,
};
use std::{
    io::Write,
    ops::Range,
    sync::mpsc::{channel, Receiver},
};

use crate::{
    renderer::RenderRange,
    tile::{Tile, Tiler},
    Dimensions, Spp,
};

use super::progress;

use image::{ImageBuffer, Rgb32FImage};
use rayon::{
    iter::{
        IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
    },
    Scope,
};
use rt::{
    camera::Camera,
    color::{ColorspaceConversion, Luma, Rgb},
    integrators::Integrator,
    memory::{Arena, ArenaInner},
    renderer::{GenericRenderResult, PixelRenderResult, RaySeries, World},
    utils::counter::counter,
    Ctx,
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

    pub allowed_error: Option<f32>,

    // TODO: make a pool of materials
    pub integrator: Box<dyn Integrator>,
    pub camera: Camera,
    pub spp: u32,

    pub seed: u64,
}

const SCRATCH_MEMORY_SIZE: usize = 1024 * 1024; // 1 MB

type Luma32FImage = image::ImageBuffer<image::Luma<f32>, Vec<f32>>;
pub type OutputBuffers = GenericRenderResult<Rgb32FImage, Luma32FImage>;

trait OutputBuffersExt<T, L> {
    fn convert(&mut self, d: GenericRenderResult<T, L>, x: u32, y: u32);
    fn new(d: Dimensions) -> Self;
}
impl OutputBuffersExt<Rgb, Luma> for OutputBuffers {
    fn convert(&mut self, d: PixelRenderResult, x: u32, y: u32) {
        *self.color.get_pixel_mut(x, y) = d.color.convert().into();
        *self.variance.get_pixel_mut(x, y) = d.variance.into();
        *self.normal.get_pixel_mut(x, y) = d.normal.convert().into();
        *self.albedo.get_pixel_mut(x, y) = d.albedo.convert().into();
        *self.position.get_pixel_mut(x, y) = d.position.convert().into();
        *self.z.get_pixel_mut(x, y) = d.z.into();
        *self.ray_depth.get_pixel_mut(x, y) = d.ray_depth.into();
    }

    fn new(Dimensions { width, height }: Dimensions) -> Self {
        Self {
            color: ImageBuffer::new(width, height),
            variance: Luma32FImage::new(width, height),
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
        world: &World,
        mut on_tile_rendered: F,
        pixel_range: RenderRange,
        sample_range: Spp,
    ) -> anyhow::Result<OutputBuffers> {
        log::debug!("Monothreaded");

        let mut output_buffers = OutputBuffers::new(self.dimension);
        let (tx, rx) = channel();
        let mut dispatcher_ = self.build_dispatcher(
            |msg| {
                tx.send(Message::Tile(msg)).unwrap();
            },
            pixel_range.x,
            pixel_range.y,
        );
        let dispatcher = &mut dispatcher_;

        let progress = match &sample_range {
            Spp::Spp(s) => progress::Progress::new(s.len() * dispatcher.tiler.tile_count()),
        };
        progress.print();

        let generation_result = rayon::scope(|s: &Scope<'_>| {
            log::info!("Generating image...");
            s.spawn(|_| {
                let output_buffers = &mut output_buffers;
                let progress = &progress;
                let rx: Receiver<Message> = rx; // Force move without moving anything else
                let mut last_progress_update = std::time::Instant::now();

                loop {
                    let Some(msg) = rx.try_recv().ok() else {
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
                        progress.print();
                        let _ = std::io::stdout().flush();
                        last_progress_update = std::time::Instant::now();
                    }
                }
                progress.print();
                println!();
                let _ = std::io::stdout().flush();
            });

            for samples in SampleCounter::new(32, sample_range) {
                dispatcher.dispatch_async(world, samples, &progress);
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
        world: &World,
        on_tile_rendered: F,
        pixel_range: RenderRange,
        samples_range: Spp,
    ) -> anyhow::Result<OutputBuffers> {
        log::debug!("Monothreaded");

        let mut output_buffers = OutputBuffers::new(self.dimension);
        let mut dispatcher = self.build_dispatcher(on_tile_rendered, pixel_range.x, pixel_range.y);
        let progress = match &samples_range {
            Spp::Spp(s) => progress::Progress::new(s.len() * dispatcher.tiler.tile_count()),
        };
        progress.print();

        log::info!("Generating image...");

        let mut arena = ArenaInner::new(SCRATCH_MEMORY_SIZE);

        for samples in SampleCounter::new(32, samples_range) {
            dispatcher.dispatch_sync(world, &mut arena, samples, &mut output_buffers, &progress);
        }
        println!();

        log::info!("Image fully generated");

        Ok(output_buffers)
    }

    fn build_dispatcher<F>(
        self,
        on_tile_rendered: F,
        x: Range<u32>,
        y: Range<u32>,
    ) -> Dispatcher<F> {
        let tiler = Tiler {
            offset_x: x.start,
            offset_y: y.start,
            width: x.len() as _,
            height: y.len() as _,
            x_grainsize: self.tile_size,
            y_grainsize: self.tile_size,
        };

        Dispatcher {
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
        }
    }

    fn tile_worker(
        &self,
        world: &World,
        arena: &mut ArenaInner,
        tile: Tile,
        data: &mut [RaySeries],
        samples: &Range<u32>,
    ) {
        assert_eq!(data.len(), tile.len());

        log::trace!("working on tile {tile:?}");
        let sqr_sample = f32::sqrt(self.spp as f32).floor() as u32;

        for (index, (x, y)) in tile.into_iter().enumerate() {
            let mut sampler = StratifiedSampler::new(x, y, sqr_sample, sqr_sample);
            // let mut sampler = UniformSampler::new(x, y);

            for sample_idx in samples.clone() {
                arena.reuse();
                sampler.with_sample(sample_idx);

                let seed = Seed {
                    x,
                    y,
                    sample_idx,
                    seed: self.seed,
                };
                let mut ctx = Ctx {
                    seed,
                    sampler: &mut sampler,
                    world,
                    rng: seed.into_rng(0),
                    arena: Arena::new(arena),
                };

                self.pixel_worker(&mut ctx, &mut data[index]);

                if let Some(allowed_error) = self.allowed_error {
                    if data[index].color.is_precise_enough(allowed_error).is_some() {
                        counter!("Adaptative sampling break");
                        break;
                    }
                }
            }
        }
    }

    fn pixel_worker(&self, ctx: &mut Ctx, res: &mut RaySeries) {
        let pcoords = ctx.sampler.sample_2d();

        let filtered_sample = BoxFilter {
            radius: Vec2::splat(0.7),
        }
        .sample(pcoords);

        let coords = Vec2 {
            x: ctx.seed.x as f32 + 0.5,
            y: ctx.seed.y as f32 + 0.5,
        } + filtered_sample.coords;

        let camera_ray = self.camera.ray(ctx, coords);
        let sample = self.integrator.ray_cast(ctx, camera_ray, 0);
        res.add_sample(sample, filtered_sample.weight);
    }
}

struct Dispatcher<F> {
    tiler: Tiler,
    tiles_data: Vec<Vec<RaySeries>>,
    on_tile_rendered: F,
    executor: Executor,
}

impl<F: FnMut(TileMsg)> Dispatcher<F> {
    fn dispatch_sync(
        &mut self,
        world: &World,
        arena: &mut ArenaInner,
        samples: Range<u32>,
        output_buffers: &mut OutputBuffers,
        progress: &progress::Progress,
    ) {
        for (tile, data) in self.tiler.into_iter().zip(self.tiles_data.iter_mut()) {
            self.executor
                .tile_worker(world, arena, tile, data, &samples);

            progress.add(samples.len() as _);
            progress.print();
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

impl<F: Fn(TileMsg) + Sync> Dispatcher<F> {
    fn dispatch_async(
        &mut self,
        world: &World,
        samples: Range<u32>,
        progress: &progress::Progress,
    ) {
        self.tiler
            .into_par_iter()
            .zip(self.tiles_data.par_iter_mut())
            .map_init(
                || ArenaInner::new(SCRATCH_MEMORY_SIZE),
                |arena, (tile, data)| {
                    self.executor
                        .tile_worker(world, arena, tile, data, &samples);
                    progress.add(samples.len() as _);

                    TileMsg {
                        tile,
                        data: data.iter().map(|x| x.as_pixelresult()).collect::<Vec<_>>(),
                    }
                },
            )
            .for_each(&self.on_tile_rendered)
    }
}

struct SampleCounter {
    batch_size: u32,
    cur: u32,
    spp: Spp,
}

impl SampleCounter {
    fn new(batch_size: u32, spp: Spp) -> Self {
        Self {
            batch_size,
            spp: spp.clone(),
            cur: match spp {
                Spp::Spp(r) => r.start,
            },
        }
    }
}

impl Iterator for SampleCounter {
    type Item = Range<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        let samples = match &mut self.spp {
            Spp::Spp(r) => {
                if Range::is_empty(r) {
                    return None;
                }
                let samples = u32::min(self.batch_size, r.len() as u32);
                *r = (r.start + samples)..r.end;
                samples
            }
        };

        let res = Some(self.cur..(self.cur + samples));
        self.cur += samples;
        res
    }
}
