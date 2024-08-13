use std::path::PathBuf;

use anyhow::{Context, Result};
use rand::{distributions::Alphanumeric, Rng};
use rt::renderer::RgbChannel;
use tev_client::{PacketCreateImage, PacketUpdateImage, TevClient};

use crate::{executor::TileMsg, Dimensions};

use super::StreamingOutput;

pub struct TevStreaming {
    client: TevClient,
    image_name: String,
    opened: bool,
    dimension: Dimensions,
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
        let client = match try_connect() {
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

        Ok(Self {
            client,
            image_name,
            opened: false,
            dimension,
        })
    }
}

impl StreamingOutput for TevStreaming {
    fn send_msg(&mut self, msg: &TileMsg) -> Result<()> {
        if msg.data.is_empty() {
            return Ok(());
        }

        assert!(msg.data.len() == msg.tile.len());

        let mut channel_names = Vec::new();
        let mut channel_offsets = Vec::<u64>::new();
        for channel in &msg.data[0].channels {
            match channel {
                rt::renderer::Channel::RgbChannel(name, _) => {
                    if *name != RgbChannel::Color {
                        channel_names.push(name.to_string() + ".X");
                        channel_names.push(name.to_string() + ".Y");
                        channel_names.push(name.to_string() + ".Z");
                    } else {
                        channel_names.push("R".into());
                        channel_names.push("G".into());
                        channel_names.push("B".into());
                    }
                    channel_offsets.push(channel_offsets.len() as _);
                    channel_offsets.push(channel_offsets.len() as _);
                    channel_offsets.push(channel_offsets.len() as _);
                }
                rt::renderer::Channel::LumaChannel(name, _) => {
                    channel_names.push(name.to_string());
                    channel_offsets.push(channel_offsets.len() as _);
                }
            }
        }
        let channel_strides = vec![channel_offsets.len() as u64; channel_offsets.len()];

        let mut data = Vec::new();
        for p in &msg.data {
            debug_assert_eq!(msg.data[0].channels.len(), p.channels.len());
            for chan in &p.channels {
                match chan {
                    rt::renderer::Channel::RgbChannel(_, c) => data.extend(c.0),
                    rt::renderer::Channel::LumaChannel(_, c) => data.push(c.0),
                }
            }
        }

        if !self.opened {
            self.client.send(PacketCreateImage {
                image_name: &self.image_name,
                grab_focus: true,
                channel_names: &channel_names,
                width: self.dimension.width,
                height: self.dimension.height,
            })?;
            self.opened = true;
        }

        self.client
            .send(PacketUpdateImage {
                image_name: &self.image_name,
                grab_focus: false,
                channel_names: &channel_names,
                channel_offsets: &channel_offsets,
                channel_strides: &channel_strides,
                x: msg.tile.x_start,
                y: msg.tile.y_start,
                width: msg.tile.width() as u32,
                height: msg.tile.height() as u32,
                data: &data,
            })
            .context("Can't send Packet to tev client. It may be closed")
    }
}
