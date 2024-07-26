#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]

mod executor;
mod output;
mod progress;
mod renderer;
mod tile;
mod utils;

use anyhow::Ok;
use clap::Parser;
use renderer::Renderer;
use rt::aggregate::embree::EmbreeScene;
use utils::{AvailableIntegrator, AvailableOutput, AvailableScene, Dimensions, Spp};

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
    tile_size: Option<u32>,

    #[arg(long, default_value_t = false)]
    disable_threading: bool,
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    let device = embree4_rs::Device::try_new(None)?;
    let scene = {
        let mut scene = EmbreeScene::new(&device);
        args.scene.insert_into(&mut scene);
        scene
    };

    let commited_scene = scene.commit()?;
    let world = commited_scene.into_world()?;

    let renderer = Renderer::from_args(args)?;

    renderer.run(&world)?;

    Ok(())
}
