use std::sync::{
    mpsc::{channel, Receiver},
    Arc,
};

use crate::Dimensions;

use super::progress;
use bytemuck::Zeroable;
use image::{ImageBuffer, Luma, Rgb32FImage};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use raytracing::{
    camera::PixelCoord,
    integrators::Integrator,
    renderer::{DefaultRenderer, GenericRenderResult, PixelRenderResult, Renderer, Channel},
};

use itertools::Itertools;
use raytracing::scene::Scene;

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

pub struct TileRendererCreateInfo {
    pub dimension: Dimensions,
    pub spp: u32,
    pub tile_size: u32,
    pub scene: Scene,
    pub shuffle_tiles: bool,
    pub integrator: Box<dyn Integrator>,
}

pub struct TileRenderer {
    pub dimension: Dimensions,
    pub tile_size: u32,
    pub shuffle_tiles: bool,
    pub renderer: Renderer,
}

type Luma32FImage = image::ImageBuffer<Luma<f32>, Vec<f32>>;
pub type OutputBuffers = GenericRenderResult<Rgb32FImage, Luma32FImage>;

impl TileRenderer {
    pub fn new(tile_create_info: TileRendererCreateInfo) -> Self {
        Self {
            dimension: tile_create_info.dimension.clone(),
            tile_size: tile_create_info.tile_size,
            shuffle_tiles: tile_create_info.shuffle_tiles,

            renderer: DefaultRenderer {
                width: tile_create_info.dimension.width,
                height: tile_create_info.dimension.height,
                spp: tile_create_info.spp,
                scene: tile_create_info.scene,
                integrator: tile_create_info.integrator,
            }
            .into(),
        }
    }

    pub fn run<F: FnMut(Arc<TileMsg>) -> () + Send>(
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
                                *output_buffers.color.get_pixel_mut(x + i, y + j) = c.to_srgb().into()
                            }
                            Channel::Normal(c) => {
                                *output_buffers.normal.get_pixel_mut(x + i, y + j) = c.to_srgb().into()
                            }
                            Channel::Albedo(c) => {
                                *output_buffers.albedo.get_pixel_mut(x + i, y + j) = c.to_srgb().into()
                            }
                            Channel::Position(c) => {
                                *output_buffers.position.get_pixel_mut(x + i, y + j) = c.to_srgb().into()
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

        let progress = progress::Progress::new((tile_count_x * tile_count_y) as usize);
        let mut generation_result = Ok(());

        rayon::scope(|s| {
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
                        progress.print();
                        last_progress_update = std::time::Instant::now();
                    }
                }
                progress.print();
            });

            // Product Tiles indices
            let mut v = (0..tile_count_x)
                .cartesian_product(0..tile_count_y)
                .collect::<Vec<_>>();

            // reorder them if needed
            if self.shuffle_tiles {
                v.shuffle(&mut thread_rng());
            }

            // Dispatch work
            generation_result = v.into_iter().par_bridge().try_for_each_with(
                tx.clone(),
                |tx, (tile_x, tile_y)| -> Result<()> {
                    let data = self.tile_worker((tile_x, tile_y));
                    progress.inc();

                    // Broadcast results to the thread which is in charge
                    tx.send(Message::Tile(Arc::new(TileMsg {
                        tile_x,
                        tile_y,
                        data,
                    })))?;

                    Ok(())
                },
            );

            tx.send(Message::Stop).unwrap();
        });

        match generation_result {
            Ok(_) => log::info!("Image fully generated"),
            Err(err) => log::info!("Image generation interrupted: {}", err),
        };

        Ok(output_buffers)
    }

    fn tile_worker(&self, (tile_x, tile_y): (u32, u32)) -> Vec<PixelRenderResult> {
        let x_range =
            (tile_x * self.tile_size)..((tile_x + 1) * self.tile_size).min(self.dimension.width);
        let y_range =
            (tile_y * self.tile_size)..((tile_y + 1) * self.tile_size).min(self.dimension.height);
        let tile_width = x_range.len();
        let tile_height = y_range.len();

        let mut data = Vec::new();
        data.resize(tile_width * tile_height, PixelRenderResult::zeroed());

        for (j, y) in y_range.clone().enumerate() {
            for (i, x) in x_range.clone().enumerate() {
                let index = j * tile_width as usize + i;
                data[index] = self.pixel_worker(PixelCoord { x, y });
            }
        }

        log::debug!("Tile {tile_x} {tile_y} done !");
        data
    }

    fn pixel_worker(&self, coords: PixelCoord) -> PixelRenderResult {
        self.renderer.process_pixel(coords)
    }
}
