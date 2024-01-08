use std::sync::mpsc::Sender;

use image::Rgba;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use raytracing::progress;
use raytracing::renderer::{DefaultRenderer, Renderer};

use itertools::Itertools;

pub struct PixelMsg {
    pub x: u32,
    pub y: u32,
    pub color: Rgba<f32>,
}

#[derive(Clone, Copy)]
pub struct GUIRenderer {
    height: u32,
    width: u32,
}

impl GUIRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { height, width }
    }

    pub fn run(&mut self, channel: Sender<PixelMsg>) {
        let width = self.width;
        let height = self.height;

        let progress = progress::Progress::new((width * height) as usize);
        rayon::scope(|s| {
            let renderer: Renderer = DefaultRenderer { width, height }.into();

            log::info!("Generating image...");
            s.spawn(|_| loop {
                if progress.updated() {
                    log::info!("{}", progress);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                } else if progress.done() {
                    return;
                }
            });

            let mut v = (0..width).cartesian_product(0..height).collect::<Vec<_>>();
            v.shuffle(&mut thread_rng());

            // Note that this will stop whenever channel is closed (Aka. the receiver channel is closed)
            let generation_result = v.into_iter().par_bridge().try_for_each_with(
                channel,
                |channel, (x, y)| -> anyhow::Result<()> {
                    // pixels in the image crate are from left to right, top to bottom
                    let vx = 2. * (x as f32 / (renderer.camera.width - 1) as f32) - 1.;
                    let vy = 1. - 2. * (y as f32 / (renderer.camera.height - 1) as f32);
                    let color = renderer.process_pixel(vx, vy).color;

                    channel.send(PixelMsg {
                        x,
                        y,
                        color: Rgba([color[0], color[1], color[2], 1.0]),
                    })?;
                    progress.inc();
                    Ok(())
                },
            );

            match generation_result {
                Ok(_) => log::info!("Image fully generated"),
                Err(err) => log::info!("Image generation interrupted: {}", err),
            };

            // To prevent progress display thread to be locked forever
            progress.set_done();
        });
    }
}
