#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]

mod cli;
mod output;
mod progress;
mod renderer;
mod storage;

use std::{fmt::Display, str::FromStr};

use clap::{Parser, ValueEnum};
use cli::Cli;
use rt::{
    integrators::{BasicIntegrator, Integrator, WhittedIntegrator},
    scene::{
        examples::{CornellBoxScene, DebugScene, DragonScene, SpheresScene, StandfordBunnyScene},
        Scene,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum Spp {
    Spp(u32),
    Inf,
}

impl FromStr for Spp {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("inf") {
            Ok(Spp::Inf)
        } else {
            Ok(Spp::Spp(s.parse()?))
        }
    }
}

#[derive(Debug, Default, Clone, Copy, ValueEnum)]
pub enum AvailableScene {
    Bunny,
    #[default]
    CornellBox,
    Spheres,
    Debug,
    Dragon,
}

impl From<AvailableScene> for Scene {
    fn from(val: AvailableScene) -> Self {
        match val {
            AvailableScene::Bunny => StandfordBunnyScene.into(),
            AvailableScene::CornellBox => CornellBoxScene.into(),
            AvailableScene::Spheres => SpheresScene.into(),
            AvailableScene::Debug => DebugScene.into(),
            AvailableScene::Dragon => DragonScene.into(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Hash)]
pub enum AvailableOutput {
    #[default]
    Tev,
    File,
}

#[derive(Default, Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Hash)]
pub enum AvailableIntegrator {
    #[default]
    Basic,
    Whitted,
}

impl From<AvailableIntegrator> for Box<dyn Integrator> {
    fn from(val: AvailableIntegrator) -> Self {
        match val {
            AvailableIntegrator::Basic => Box::new(BasicIntegrator { max_depth: 64 }),
            AvailableIntegrator::Whitted => Box::new(WhittedIntegrator { max_depth: 64 }),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Dimensions {
    width: u32,
    height: u32,
}

impl std::str::FromStr for Dimensions {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split_it = s.split('x');
        let (Some(a), Some(b)) = (split_it.next(), split_it.next()) else {
            return Err(anyhow::anyhow!("Incorrect format, see help"));
        };
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
    #[arg(long = "spp", default_value = "1")]
    /// Samples per pixels
    sample_per_pixel: Spp,

    #[arg(long, value_enum, default_value_t)]
    /// Scene selector
    scene: AvailableScene,

    #[arg(short, long, default_value = "800x600")]
    /// Screen dimension in format `width`x`height`
    dimensions: Dimensions,

    #[arg(short, long, value_enum)]
    output: Vec<AvailableOutput>,

    #[arg(short, long, value_enum)]
    integrator: AvailableIntegrator,

    #[arg(long)]
    tev_hostname: Option<String>,

    /// If provided, allow for a kind of adaptative sampling by estimating the error of a pixel until the error if less than the given value
    #[arg(long)]
    allowed_error: Option<f32>,

    #[arg(long)]
    no_threads: bool,

    #[arg(long)]
    tile_size: Option<u32>,
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    Cli::new(args)?.run()
}
