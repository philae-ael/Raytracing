mod cli;
mod output;
mod tile_renderer;
pub mod progress;

use std::fmt::Display;

use clap::{Parser, ValueEnum};
use cli::Cli;
use raytracing::scene::{
    examples::{CornellBoxScene, StandfordBunnyScene},
    Scene,
};

#[derive(Debug, Default, Clone, Copy, ValueEnum)]
pub enum AvailableScene {
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

#[derive(Default, Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Hash)]
pub enum AvailableOutput {
    #[default]
    Tev,
    Vulkan,
    File,
}

#[derive(Clone, Debug)]
pub struct Dimensions {
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
pub struct Args {
    tev_path: Option<String>,
    #[arg(long = "spp", default_value_t = 1)]
    /// Samples per pixels
    sample_per_pixel: u32,

    #[arg(long, value_enum, default_value_t)]
    /// Scene selector
    scene: AvailableScene,

    #[arg(short, long, default_value = "800x600")]
    /// Screen dimension in format `width`x`height`
    dimensions: Dimensions,

    #[arg(short, long, value_enum)]
    output: Vec<AvailableOutput>,

    #[arg(long)]
    tev_hostname: Option<String>,
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    Cli::new(args)?.run()
}
