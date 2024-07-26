use core::fmt::Display;
use std::str::FromStr;

use clap::ValueEnum;
use rt::{
    integrators::{BasicIntegrator, Integrator, WhittedIntegrator},
    scene::{
        examples::{CornellBoxScene, DebugScene, DragonScene, SpheresScene, StandfordBunnyScene},
        SceneT,
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

impl AvailableScene {
    pub fn insert_into(self, scene: &mut impl SceneT) {
        match self {
            AvailableScene::Bunny => StandfordBunnyScene::insert_into(scene),
            AvailableScene::CornellBox => CornellBoxScene::insert_into(scene),
            AvailableScene::Spheres => SpheresScene::insert_into(scene),
            AvailableScene::Debug => DebugScene::insert_into(scene),
            AvailableScene::Dragon => DragonScene::insert_into(scene),
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
    pub width: u32,
    pub height: u32,
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
