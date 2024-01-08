use std::sync::mpsc::{channel, Receiver};

use bytemuck::Zeroable;
use rand::distributions::Alphanumeric;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rayon::prelude::{ParallelBridge, ParallelIterator};
use raytracing::progress;
use raytracing::renderer::{DefaultRenderer, RenderResult, Renderer};

use itertools::Itertools;
use tev_client::{PacketCreateImage, PacketUpdateImage, TevClient};

pub struct TileMsg {
    pub tile_x: u32,
    pub tile_y: u32,
    pub data: Vec<RenderResult>,
}

#[derive(Clone, Copy)]
pub struct TevRenderer {
    pub height: u32,
    pub width: u32,
    pub tile_size: u32,
}

impl TevRenderer {
    fn get_id() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect()
    }

    pub fn run(&mut self, mut client: TevClient) -> anyhow::Result<()> {
        let width = self.width;
        let height = self.height;
        let tile_size = self.tile_size;

        // assert!(
        // width % tile_size == 0 && height % tile_size == 0,
        // "Cant split image in even tiles"
        // );

        let tile_count_x = (width as f32 / tile_size as f32).ceil() as u32;
        let tile_count_y = (height as f32 / tile_size as f32).ceil() as u32;

        let image_name = format!("raytraced-{}", Self::get_id());

        let channel_names = [
            "R", "G", "B", // color
            "normal.X", "normal.Y", "normal.Z", // normal
            "albedo.R", "albedo.G", "albedo.B", // albedo
            "Z",        // depth
        ];
        let channel_offset = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let channel_stride = [10; 10];

        client.send(PacketCreateImage {
            image_name: &image_name,
            grab_focus: true,
            channel_names: &channel_names,
            width,
            height,
        })?;

        let progress = progress::Progress::new((tile_count_x * tile_count_y) as usize);
        let mut generation_result = Ok(());

        rayon::scope(|s| {
            let renderer: Renderer = DefaultRenderer { width, height }.into();
            let (tx, rx) = channel();

            log::info!("Generating image...");
            s.spawn(|_| {
                let rx: Receiver<TileMsg> = rx; // Force move without moving anything else
                for msg in rx.iter() {
                    let x = msg.tile_x * tile_size;
                    let y = msg.tile_y * tile_size;
                    let tile_width = (x + tile_size).min(width) - x;
                    let tile_height = (y + tile_size).min(height) - y;

                    assert!(msg.data.len() == (tile_width * tile_height) as usize);
                    let data = bytemuck::cast_slice(msg.data.as_slice());

                    client
                        .send(PacketUpdateImage {
                            image_name: &image_name,
                            grab_focus: false,
                            channel_names: &channel_names,
                            channel_offsets: &channel_offset,
                            channel_strides: &channel_stride,
                            x,
                            y,
                            width: tile_width,
                            height: tile_height,
                            data,
                        })
                        .unwrap();
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
