use std::{fmt::Display, path::PathBuf};

use clap::{Parser, ValueEnum};
use raytracing::scene::{
    examples::{CornellBoxScene, StandfordBunnyScene},
    Scene,
};
use tev_client::TevClient;
use tev_renderer::TevRenderer;

mod tev_renderer;

#[derive(Debug, Default, Clone, Copy, ValueEnum)]
enum AvailableScene {
    Bunny,
    #[default]
    CornellBox,
}

impl Into<Scene> for AvailableScene {
    fn into(self) -> Scene {
        match self {
            AvailableScene::Bunny => StandfordBunnyScene.into(),
            AvailableScene::CornellBox => CornellBoxScene.into(),
        }
    }
}

#[derive(Clone, Debug)]
struct Dimensions {
    width: u32,
    height: u32,
}

impl std::str::FromStr for Dimensions {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split_it = s.split("x");
        let (Some(a), Some(b)) = (split_it.next(), split_it.next()) else {return Err(anyhow::anyhow!("Incorrect format, see help"));};
        let width: u32 = a.parse()?;
        let height: u32 = b.parse()?;

        Ok(Dimensions { width, height })
    }
}

impl Display for Dimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}x{}", self.width, self.height))
    }
}

#[derive(Parser, Debug)]
struct Args {
    tev_path: Option<PathBuf>,
    #[arg(long = "spp", default_value_t = 20)]
    /// Samples per pixels
    sample_per_pixel: u32,

    #[arg(long, value_enum, default_value_t)]
    /// Scene selector
    scene: AvailableScene,

    #[arg(short, long, default_value = "800x600")]
    /// Screen dimension in format `width`x`height`
    dimensions: Dimensions,
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    let client = if let Some(tev_path) = args.tev_path {
        let command = std::process::Command::new(tev_path);
        TevClient::spawn(command)
    } else {
        Ok(TevClient::wrap(std::net::TcpStream::connect(
            "127.0.0.1:14158",
        )?))
    }?;
    log::info!(
        "Will run a rendering of scene {:?} on a screen of dimension {} with {} samples per pixels",
        args.scene,
        args.dimensions,
        args.sample_per_pixel
    );

    TevRenderer {
        width: args.dimensions.width,
        height: args.dimensions.height,
        spp: args.sample_per_pixel,
        tile_size: 20,
        scene: args.scene.into(),
    }
    .run(client)?;
    log::info!("Done");
    Ok(())
}
