use std::sync::mpsc::Sender;

use image::Rgba;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use renderer::progress;
use renderer::renderer::{DefaultRenderer, Renderer};

use itertools::Itertools;

pub struct PixelMsg {
    pub x: u32,
    pub y: u32,
    pub color: Rgba<f32>,
}

pub struct GUIRenderer {
    channel: Sender<PixelMsg>,
}

impl GUIRenderer {
    pub fn new(channel: Sender<PixelMsg>) -> Self {
        Self { channel }
    }

    pub fn run(&mut self) {
        let width = 500;
        let height = 500;

        let progress = progress::Progress::new((width * height) as usize);
        let channel = self.channel.clone();
        rayon::scope(|s| {
            let renderer: Renderer = DefaultRenderer { width, height }.into();

            log::info!("Generating image...");
            s.spawn(|_| {
                while !progress.done() {
                    if progress.updated() {
                        log::info!("{}", progress);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });

            let mut v = (0..width).cartesian_product(0..height).collect::<Vec<_>>();
            v.shuffle(&mut thread_rng());

            v.into_iter()
                .par_bridge()
                .for_each_with(channel, |channel, (x, y)| {
                    // pixels in the image crate are from left to right, top to bottom
                    let vx = 2. * (x as f32 / (renderer.camera.width - 1) as f32) - 1.;
                    let vy = 1. - 2. * (y as f32 / (renderer.camera.height - 1) as f32);
                    let color = renderer.process_pixel(vx, vy).color;
                    channel
                        .send(PixelMsg {
                            x,
                            y,
                            color: Rgba([color.0[0], color.0[1], color.0[2], 1.0]),
                        })
                        .unwrap();
                    progress.inc();
                });
        });
    }
}
