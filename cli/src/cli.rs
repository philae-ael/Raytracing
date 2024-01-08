use std::collections::HashSet;

use anyhow::Result;

use crate::{
    tev::TevOutput,
    tile_renderer::{TileMsg, TileRenderer},
    Args, AvailableOutput, AvailableScene, Dimensions,
};

pub trait Output: Send {
    fn send_msg(&mut self, cli: &Cli, msg: &TileMsg) -> Result<()>;
}

pub struct Cli {
    pub outputs: Vec<Box<dyn Output>>,
    pub dimensions: Dimensions,
    pub tile_size: u32,
    pub sample_per_pixel: u32,
    pub scene: AvailableScene,
}

impl Cli {
    pub fn new(args: Args) -> Result<Self> {
        let outputs: HashSet<AvailableOutput> = HashSet::from_iter(args.output.into_iter());

        let mut this = Self {
            outputs: Vec::new(),
            dimensions: args.dimensions.clone(),
            tile_size: 20,
            sample_per_pixel: args.sample_per_pixel,
            scene: args.scene,
        };

        if outputs.contains(&AvailableOutput::Tev) {
            this.outputs
                .push(Box::new(TevOutput::new(&this, args.tev_path)?));
        }

        Ok(this)
    }

    pub fn run(mut self) -> Result<()> {
        let mut outputs = Vec::new();

        TileRenderer {
            width: self.dimensions.width,
            height: self.dimensions.height,
            spp: self.sample_per_pixel,
            tile_size: 20,
            scene: self.scene.into(),
        }
        .run(move |msg| {
            // Move tev_cli out of self, work with it and move it back in self
            std::mem::swap(&mut self.outputs, &mut outputs);

            for mut output in outputs.drain(..) {
                match output.send_msg(&self, &msg) {
                    Ok(_) => self.outputs.push(output),
                    Err(err) => {
                        log::error!("{err}");
                    }
                }
            }
        })?;
        log::info!("Done");
        Ok(())
    }
}
