use std::path::PathBuf;

use anyhow::{Context, Result};
use rand::{distributions::Alphanumeric, Rng};
use tev_client::{PacketCreateImage, PacketUpdateImage, TevClient};

use crate::{executor::TileMsg, Dimensions};

use super::StreamingOutput;

const CHANNEL_COUNT: usize = 15;
fn channel_names() -> [&'static str; CHANNEL_COUNT] {
    [
        "R",
        "G",
        "B", // color
        "Variance",
        "normal.X",
        "normal.Y",
        "normal.Z", // normal
        "position.X",
        "position.Y",
        "position.Z", // position
        "albedo.R",
        "albedo.G",
        "albedo.B", // albedo
        "Z",        // depth
        "ray_depth",
    ]
}
fn channel_offsets() -> [u64; CHANNEL_COUNT] {
    core::array::from_fn(|i| i as u64)
}
fn channel_strides() -> [u64; CHANNEL_COUNT] {
    [CHANNEL_COUNT as u64; CHANNEL_COUNT]
}

pub struct TevStreaming {
    client: TevClient,
    image_name: String,
}

impl TevStreaming {
    pub fn new(
        dimension: Dimensions,
        tev_path: Option<String>,
        tev_hostname: Option<String>,
    ) -> Result<Self> {
        let tev_hostname: String = tev_hostname.unwrap_or("127.0.0.1:14158".into());
        let tev_path: String = tev_path.unwrap_or("./tev".into());

        let try_spawn = |path: PathBuf| -> Result<()> {
            let mut command = std::process::Command::new(path);
            command.arg(format!("--hostname={:?}", tev_hostname));
            command
                .stdout(std::process::Stdio::null())
                .stdin(std::process::Stdio::null())
                .spawn()?;

            // Wait for exe to be up
            // May not work
            std::thread::sleep(std::time::Duration::from_secs(2));
            Ok(())
        };
        let try_connect = || -> Result<TevClient> {
            Ok(TevClient::wrap(std::net::TcpStream::connect(
                &tev_hostname,
            )?))
        };

        log::debug!("Trying tev direct connection");
        let mut client = match try_connect() {
            Ok(client) => client,
            Err(_) => {
                log::warn!("Can't find tev client, trying to spawn tev");
                try_spawn(tev_path.into())?;
                try_connect()?
            }
        };
        log::info!("Successfully connected to tev");

        fn get_id() -> String {
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(7)
                .map(char::from)
                .collect()
        }
        let image_name = format!("raytraced-{}", get_id());

        client.send(PacketCreateImage {
            image_name: &image_name,
            grab_focus: true,
            channel_names: &channel_names(),
            width: dimension.width,
            height: dimension.height,
        })?;

        Ok(Self { client, image_name })
    }
}

impl StreamingOutput for TevStreaming {
    fn send_msg(&mut self, msg: &TileMsg) -> Result<()> {
        assert!(msg.data.len() == msg.tile.len());

        let data = bytemuck::cast_slice(msg.data.as_slice());

        self.client
            .send(PacketUpdateImage {
                image_name: &self.image_name,
                grab_focus: false,
                channel_names: &channel_names(),
                channel_offsets: &channel_offsets(),
                channel_strides: &channel_strides(),
                x: msg.tile.x_start,
                y: msg.tile.y_start,
                width: msg.tile.width() as u32,
                height: msg.tile.height() as u32,
                data,
            })
            .context("Can't send Packet to tev client. It may be closed")
    }
}
