#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]

mod executor;
mod output;
mod progress;
mod renderer;
mod tile;
mod utils;

use anyhow::Result;
use clap::Parser;
use progress::PercentBar;
use renderer::{ExecutionMode, RenderRange, Renderer};
use rt::aggregate::embree::EmbreeScene;
use utils::{AvailableIntegrator, AvailableOutput, AvailableScene, Dimensions, Spp};

#[derive(Parser, Debug)]
pub struct Args {
    tev_path: Option<String>,
    #[arg(long = "spp", default_value = "32")]
    /// Samples per pixel. To render a pixel using 5 samples use "5" to render a pixel with samples
    /// 7..84 use "7..84" to render a pixel with as much sample as possible (it will render until
    ///   interuption) use "inf"
    spp: u32,

    #[arg(long)]
    sample_range: Option<Spp>,

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

    #[arg(long, default_value_t = 32)]
    tile_size: u32,

    #[arg(short, long, value_enum, default_value_t=ExecutionMode::Multithreaded)]
    execution_mode: ExecutionMode,

    #[arg(short, long)]
    /// The range to render. To render pixel (1,4) use "1x4",to render range (1,4)..(7,45) use
    /// "1..7x4..45".
    range: Option<RenderRange>,

    #[arg(long, default_value_t)]
    /// Seed to use for all the random stuff.
    /// Given a seed, the rendering is deterministic (the output only depends on x, y, sample and seed).
    seed: u64,
}

fn build_device() -> Result<embree4_rs::device::Device> {
    log::info!("building embree device");
    let device = embree4_rs::device::Device::try_new(None)?;

    std::mem::forget(device.register_error_callback(|code, err| {
        log::error!(target:"embree", "Embree error ({code:?}): {err}");
    }));

    std::mem::forget(
        device.register_device_memory_monitor_callback(|amount, _post| {
            log::debug!(target:"embree", "allocation {amount}bytes");
            true
        }),
    );

    Ok(device)
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    let device = build_device()?;

    log::info!("loading scene");
    let mut scene = EmbreeScene::new(&device);
    args.scene.insert_into(&mut scene);

    log::info!("building scene");
    let commited_scene = scene.commit_with_progress(|amount| {
        // log::info!(target:"embree::scene", "progress: {}%", amount*100);
        print!(
            "\r{}",
            PercentBar {
                percent: amount as _,
                width: 50
            }
        );
        true
    })?;
    println!();

    let world = commited_scene.into_world()?;

    let renderer = Renderer::from_args(args)?;
    renderer.run(&world)?;

    Ok(())
}
