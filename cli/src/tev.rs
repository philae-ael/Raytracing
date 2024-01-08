use std::path::PathBuf;

use anyhow::{Result, Context};
use rand::{distributions::Alphanumeric, Rng};
use tev_client::{PacketCreateImage, PacketUpdateImage, TevClient};

use crate::{
    cli::{Cli, Output},
    tile_renderer::TileMsg,
};

pub struct TevOutput {
    client: TevClient,
    image_name: String,
    channel_names: [&'static str; 11],
    channel_offsets: [u64; 11],
    channel_strides: [u64; 11],
}

impl TevOutput {
    pub fn new(cli: &Cli, tev_path: Option<PathBuf>) -> Result<Self> {
        let mut client = if let Some(tev_path) = tev_path {
            let command = std::process::Command::new(tev_path);
            TevClient::spawn(command)
        } else {
            Ok(TevClient::wrap(std::net::TcpStream::connect(
                "127.0.0.1:14158",
            )?))
        }?;

        fn get_id() -> String {
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(7)
                .map(char::from)
                .collect()
        }
        let image_name = format!("raytraced-{}", get_id());
        let channel_names = [
            "R",
            "G",
            "B", // color
            "normal.X",
            "normal.Y",
            "normal.Z", // normal
            "albedo.R",
            "albedo.G",
            "albedo.B", // albedo
            "Z",        // depth
            "ray_depth",
        ];

        let channel_offsets = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let channel_strides = [11; 11];

        client.send(PacketCreateImage {
            image_name: &image_name,
            grab_focus: true,
            channel_names: &channel_names,
            width: cli.dimensions.width,
            height: cli.dimensions.height,
        })?;

        Ok(Self {
            client,
            image_name,
            channel_names,
            channel_offsets,
            channel_strides,
        })
    }
}

impl Output for TevOutput {
    fn send_msg(&mut self, cli: &Cli, msg: &TileMsg) -> Result<()> {
        let x = msg.tile_x * cli.tile_size;
        let y = msg.tile_y * cli.tile_size;
        let tile_width = (x + cli.tile_size).min(cli.dimensions.width) - x;
        let tile_height = (y + cli.tile_size).min(cli.dimensions.height) - y;

        assert!(msg.data.len() == (tile_width * tile_height) as usize);
        let data = bytemuck::cast_slice(msg.data.as_slice());

        self.client
            .send(PacketUpdateImage {
                image_name: &self.image_name,
                grab_focus: false,
                channel_names: &self.channel_names,
                channel_offsets: &self.channel_offsets,
                channel_strides: &self.channel_strides,
                x,
                y,
                width: tile_width,
                height: tile_height,
                data,
            })
            .context("Can't send Packet to tev client. It may be closed")
    }
}
