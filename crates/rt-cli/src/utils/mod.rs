use core::fmt::Display;
use std::{ops::Range, str::FromStr};

use crate::Args;
use clap::ValueEnum;
use rt::{
    camera::Camera,
    integrators::{Integrator, PathTracer, RandomWalkIntegrator},
    math::{point::Point, quaternion::LookAt, vec::Vec3},
    scene::{
        examples::{CornellBoxScene, DebugScene, DragonScene, SpheresScene, StandfordBunnyScene},
        SceneT,
    },
};

pub(crate) trait FromArgs {
    fn from_args(args: &Args) -> Self;
}

#[derive(Debug, Clone, ValueEnum, Copy)]
pub enum ExecutionMode {
    Multithreaded,
    Monothreaded,
}

impl FromArgs for Camera {
    fn from_args(args: &Args) -> Self {
        let look_at = Point::new(0.0, 0.0, -1.0);
        let look_from = Point::ORIGIN;
        let look_direction = look_at - look_from;
        Camera::new(
            args.dimensions.width,
            args.dimensions.height,
            f32::to_radians(70.),
            look_direction.length(),
            look_from,
            LookAt {
                direction: look_direction,
                forward: Vec3::NEG_Z,
            }
            .into(),
            0.0,
        )
    }
}

#[derive(Debug, Clone)]
pub struct RenderRange {
    pub x: Range<u32>,
    pub y: Range<u32>,
}

impl FromArgs for RenderRange {
    fn from_args(args: &Args) -> Self {
        args.range.clone().unwrap_or(RenderRange {
            x: 0..args.dimensions.width,
            y: 0..args.dimensions.height,
        })
    }
}

impl std::str::FromStr for RenderRange {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let err = Err("expected monothreaded, multithreaded or a simple pixel `x`x`y`x`sample` eg 1x2x4 for the pixel 1 2 at sample 4");
        let mut it = s.split('x').flat_map(|x| {
            let Some(x) = x.split_once("..") else {
                return x.parse::<u32>().ok().map(|x| x..(x + 1));
            };

            Some(x.0.parse().ok()?..x.1.parse().ok()?)
        });

        let Some(x) = it.next() else {
            return err;
        };
        let Some(y) = it.next() else {
            return err;
        };
        let None = it.next() else {
            return err;
        };

        Ok(RenderRange { x, y })
    }
}

#[derive(Debug, Clone)]
pub enum Spp {
    Spp(Range<u32>),
}
impl FromArgs for Spp {
    fn from_args(args: &Args) -> Self {
        args.sample_range.clone().unwrap_or(Spp::Spp(0..args.spp))
    }
}

impl FromStr for Spp {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = match s.split_once("..") {
            Some((a, b)) => a.parse()?..b.parse()?,
            None => 0..s.parse()?,
        };
        Ok(Spp::Spp(r))
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
    Basic,
    #[default]
    PathTracer,
}

impl FromArgs for Box<dyn Integrator> {
    fn from_args(args: &Args) -> Self {
        let max_depth = args.max_ray_depth.unwrap_or(64);
        match args.integrator {
            AvailableIntegrator::Basic => Box::new(RandomWalkIntegrator { max_depth }),
            AvailableIntegrator::PathTracer => Box::new(PathTracer { max_depth }),
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
