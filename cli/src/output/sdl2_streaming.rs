use anyhow::Context;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
};
use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    time::Duration,
};

use crate::{cli::StreamingOutput, renderer::TileMsg, Dimensions};

enum Message {
    TileMsg(Arc<TileMsg>),
    Done,
}

pub struct SDL2Streaming {
    sender: Sender<Message>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl SDL2Streaming {
    pub fn new(dim: Dimensions, tile_size: u32) -> Self {
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(move || SDL2Streaming::run_inner(dim, tile_size, rx));

        Self {
            sender: tx,
            handle: Some(handle),
        }
    }
}

impl Drop for SDL2Streaming {
    fn drop(&mut self) {
        let _ = self.sender.send(Message::Done);
        let _ = self.handle.take().unwrap().join();
    }
}

impl SDL2Streaming {
    fn run_inner(dim: Dimensions, tile_size: u32, rx: Receiver<Message>) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("sdl2 interface", dim.width, dim.height)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let texture_creator = canvas.texture_creator();
        let mut text = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, dim.width, dim.height)
            .unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();
        loop {
            canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => return,
                    _ => {}
                }
            }
            for msg in rx.try_iter() {
                match msg {
                    Message::TileMsg(msg) => {
                        let x = msg.tile_x * tile_size;
                        let y = msg.tile_y * tile_size;
                        let tile_width = (x + tile_size).min(dim.width) - x;
                        let tile_height = (y + tile_size).min(dim.height) - y;
                        let rect = Rect::new(x as i32, y as i32, tile_width, tile_height);

                        let data: Vec<[u8; 3]> = msg
                            .data
                            .iter()
                            .map(|x| x.color.to_srgb().to_byte_array())
                            .collect();

                        let data = bytemuck::cast_slice(data.as_slice());
                        text.update(rect, data, 3 * tile_width as usize).unwrap();
                    }
                    Message::Done => canvas.window_mut().raise(),
                }
            }

            canvas.copy(&text, None, None).unwrap();
            canvas.present();
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}

impl StreamingOutput for SDL2Streaming {
    fn send_msg(&mut self, msg: std::sync::Arc<TileMsg>) -> anyhow::Result<()> {
        self.sender
            .send(Message::TileMsg(msg))
            .context("Could not send data to SDL2 thread, window is closed?")?;
        Ok(())
    }
}
