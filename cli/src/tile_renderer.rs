use std::sync::mpsc::{channel, Receiver};

use super::progress;
use bytemuck::Zeroable;
use image::{ImageBuffer, Luma, Rgb, Rgb32FImage};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use raytracing::renderer::{DefaultRenderer, RenderResult, Renderer};

use itertools::Itertools;
use raytracing::scene::Scene;

pub struct TileMsg {
    pub tile_x: u32,
    pub tile_y: u32,
    pub data: Vec<RenderResult>,
}

impl TileMsg {
    fn ss(&self, tile_size: u32, height: u32, width: u32) -> (u32, u32, u32, u32) {
        let x = self.tile_x * tile_size;
        let y = self.tile_y * tile_size;
        let tile_width = (x + tile_size).min(width) - x;
        let tile_height = (y + tile_size).min(height) - y;
        (x, y, tile_width, tile_height)
    }
}

pub struct TileRenderer {
    pub height: u32,
    pub width: u32,
    pub spp: u32,
    pub tile_size: u32,
    pub scene: Scene,
}

pub struct OutputBuffers {
    pub color: Rgb32FImage,
    pub normal: Rgb32FImage,
    pub albedo: Rgb32FImage,
    pub depth: image::ImageBuffer<Luma<f32>, Vec<f32>>,
    pub ray_depth: image::ImageBuffer<Luma<f32>, Vec<f32>>,
}

impl TileRenderer {
    pub fn run<F: FnMut(&TileMsg) -> () + Send>(
        self,
        on_tile_rendered: F,
    ) -> anyhow::Result<OutputBuffers> {
        let width = self.width;
        let height = self.height;
        let tile_size = self.tile_size;

        let mut output_buffers = OutputBuffers {
            color: ImageBuffer::new(width, height),
            normal: ImageBuffer::new(width, height),
            albedo: ImageBuffer::new(width, height),
            depth: ImageBuffer::new(width, height),
            ray_depth: ImageBuffer::new(width, height),
        };

        let mut push_tile_on_output_buffers = |msg: &TileMsg| {
            let (x, y, width, height) = msg.ss(self.tile_size, self.height, self.width);

            for i in 0..width {
                for j in 0..height {
                    let data_index = (i + width * j) as usize;
                    let RenderResult {
                        color,
                        normal,
                        albedo,
                        z,
                        ray_depth,
                    } = msg.data[data_index];

                    *output_buffers.ray_depth.get_pixel_mut(x + i, y + j) = Luma([ray_depth]);
                    *output_buffers.depth.get_pixel_mut(x + i, y + j) = Luma([z]);
                    *output_buffers.normal.get_pixel_mut(x + i, y + j) = Rgb(normal);
                    *output_buffers.albedo.get_pixel_mut(x + i, y + j) = Rgb(albedo);
                    *output_buffers.color.get_pixel_mut(x + i, y + j) = Rgb(color);
                }
            }
        };

        let tile_count_x = (width as f32 / tile_size as f32).ceil() as u32;
        let tile_count_y = (height as f32 / tile_size as f32).ceil() as u32;

        let progress = progress::Progress::new((tile_count_x * tile_count_y) as usize);
        let mut generation_result = Ok(());

        enum Message {
            Tile(TileMsg),
            Stop,
        }

        rayon::scope(|s| {
            let renderer: Renderer = DefaultRenderer {
                width,
                height,
                spp: self.spp,
                scene: self.scene,
            }
            .into();
            let (tx, rx) = channel();

            log::info!("Generating image...");
            s.spawn(|_| {
                let mut on_tile_rendered = on_tile_rendered;
                let rx: Receiver<Message> = rx; // Force move without moving anything else
                for msg in rx.iter() {
                    match msg {
                        Message::Tile(tile_msg) => {
                            push_tile_on_output_buffers(&tile_msg);
                            on_tile_rendered(&tile_msg);
                            progress.print();
                        }
                        Message::Stop => {
                            break;
                        }
                    }
                }
                progress.print();
            });

            let mut v = (0..tile_count_x)
                .cartesian_product(0..tile_count_y)
                .collect::<Vec<_>>();
            v.shuffle(&mut thread_rng());

            // Note that this will stop whenever channel is closed (Aka. the receiver channel is closed)
            generation_result = v.into_iter().par_bridge().try_for_each_with(
                tx.clone(),
                |tx, (tile_x, tile_y)| -> anyhow::Result<()> {
                    let x_range = (tile_x * tile_size)..((tile_x + 1) * tile_size).min(width);
                    let y_range = (tile_y * tile_size)..((tile_y + 1) * tile_size).min(height);
                    let tile_width = x_range.len();
                    let tile_height = y_range.len();

                    let mut data = Vec::new();
                    data.resize(tile_width * tile_height, RenderResult::zeroed());

                    for (i, x) in x_range.enumerate() {
                        for (j, y) in y_range.clone().enumerate() {
                            // pixels in the image crate are from left to right, top to bottom
                            let vx = 2. * (x as f32 / (renderer.camera.width - 1) as f32) - 1.;
                            let vy = 1. - 2. * (y as f32 / (renderer.camera.height - 1) as f32);
                            let index = j * tile_width as usize + i;
                            data[index] = renderer.process_pixel(vx, vy);
                        }
                    }

                    log::debug!("Tile {tile_x} {tile_y} done !");
                    tx.send(Message::Tile(TileMsg {
                        tile_x,
                        tile_y,
                        data,
                    }))?;
                    progress.inc();
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
}
