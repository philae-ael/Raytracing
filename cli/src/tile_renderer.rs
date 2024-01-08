use std::sync::mpsc::{channel, Receiver};

use bytemuck::Zeroable;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use raytracing::progress;
use raytracing::renderer::{DefaultRenderer, RenderResult, Renderer};

use itertools::Itertools;
use raytracing::scene::Scene;

pub struct TileMsg {
    pub tile_x: u32,
    pub tile_y: u32,
    pub data: Vec<RenderResult>,
}

pub struct TileRenderer {
    pub height: u32,
    pub width: u32,
    pub spp: u32,
    pub tile_size: u32,
    pub scene: Scene,
}

impl TileRenderer {
    pub fn run<F: FnMut(TileMsg) -> () + Send>(self, on_tile_rendered: F) -> anyhow::Result<()> {
        let width = self.width;
        let height = self.height;
        let tile_size = self.tile_size;

        // assert!(
        // width % tile_size == 0 && height % tile_size == 0,
        // "Cant split image in even tiles"
        // );

        let tile_count_x = (width as f32 / tile_size as f32).ceil() as u32;
        let tile_count_y = (height as f32 / tile_size as f32).ceil() as u32;

        let progress = progress::Progress::new((tile_count_x * tile_count_y) as usize);
        let mut generation_result = Ok(());

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
                let rx: Receiver<TileMsg> = rx; // Force move without moving anything else
                for msg in rx.iter() {
                    on_tile_rendered(msg);
                    progress.print();
                }
            });

            let mut v = (0..tile_count_x)
                .cartesian_product(0..tile_count_y)
                .collect::<Vec<_>>();
            v.shuffle(&mut thread_rng());

            // Note that this will stop whenever channel is closed (Aka. the receiver channel is closed)
            generation_result = v.into_iter().par_bridge().try_for_each_with(
                tx,
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
                    tx.send(TileMsg {
                        tile_x,
                        tile_y,
                        data,
                    })?;
                    progress.inc();
                    Ok(())
                },
            );

            // To prevent progress display thread to be locked forever on abrupt interruptions
            progress.set_done();
        });

        match generation_result {
            Ok(_) => log::info!("Image fully generated"),
            Err(err) => log::info!("Image generation interrupted: {}", err),
        };

        Ok(())
    }
}
