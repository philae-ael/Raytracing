use derive_more::derive::Display;

use crate::{
    color::{self, Luma, Rgb},
    material::{MaterialDescriptor, MaterialId},
    math::{
        point::Point,
        stat::{FilteredRgb, RgbSeries},
        vec::{RgbAsVec3Ext, Vec3, Vec3AsRgbExt},
    },
    shape::Shape,
};

pub struct RayResult {
    pub normal: Vec3,
    pub position: Point,
    pub albedo: Rgb,
    pub color: Rgb,
    pub z: f32,
    pub ray_depth: f32,
    pub samples_accumulated: u32,
}

#[derive(Clone, Default)]
pub struct RaySeries {
    pub samples_accumulated: u32,
    pub color: RgbSeries,
    pub filtered_color: FilteredRgb,
    pub position: Point,
    pub normal: Vec3,
    pub albedo: Rgb,
    pub ray_depth: f32,
    pub z: f32,
}

impl RaySeries {
    pub fn as_pixelresult(&self) -> PixelRenderResult {
        let RaySeries {
            position,
            normal,
            albedo,
            color,
            filtered_color,
            z,
            ray_depth,
            samples_accumulated,
        } = self;

        let inv_samples = 1.0 / *samples_accumulated as f32;
        PixelRenderResult {
            channels: vec![
                RgbChannel::Normal.channel((inv_samples * *normal).rgb()),
                RgbChannel::Position.channel((inv_samples * position.vec()).rgb()),
                RgbChannel::Albedo.channel((inv_samples * albedo.vec()).rgb()),
                RgbChannel::Color.channel(filtered_color.value()),
                LumaChannel::Variance.channel(color.variance()),
                LumaChannel::Z.channel(color::Luma(inv_samples * z)),
                LumaChannel::RayDepth.channel(color::Luma(inv_samples * ray_depth)),
            ],
        }
    }

    pub fn add_sample(&mut self, rhs: RayResult, weight: f32) {
        let RayResult {
            normal,
            position,
            albedo,
            color,
            z,
            ray_depth,
            samples_accumulated,
        } = rhs;

        self.color.add_sample(color);
        self.filtered_color.add_sample(color, weight);
        self.normal += normal;
        self.position = Point(self.position.vec() + position.vec());
        self.albedo = (self.albedo.vec() + albedo.vec()).rgb();
        self.z += z;
        self.ray_depth += ray_depth;
        self.samples_accumulated += samples_accumulated;
    }

    pub fn merge(lhs: Self, rhs: Self) -> Self {
        Self {
            normal: lhs.normal + rhs.normal,
            position: Point(lhs.position.vec() + rhs.position.vec()),
            albedo: (lhs.albedo.vec() + rhs.albedo.vec()).rgb(),
            color: RgbSeries::merge(lhs.color, rhs.color),
            filtered_color: FilteredRgb::merge(lhs.filtered_color, rhs.filtered_color),
            z: lhs.z + rhs.z,
            ray_depth: lhs.ray_depth + rhs.ray_depth,
            samples_accumulated: lhs.samples_accumulated + rhs.samples_accumulated,
        }
    }
}

impl Default for RayResult {
    fn default() -> Self {
        Self {
            normal: color::linear::BLACK.vec(),
            position: Point::ORIGIN,
            albedo: color::linear::BLACK,
            color: color::linear::BLACK,
            z: 0.0,
            ray_depth: 0.0,
            samples_accumulated: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Display, PartialEq, Eq)]
pub enum RgbChannel {
    Color,
    Position,
    Albedo,
    Normal,
}

impl RgbChannel {
    pub fn channel<RgbStorage, LumaStorage>(
        self,
        rgb: RgbStorage,
    ) -> Channel<RgbStorage, LumaStorage> {
        Channel::RgbChannel(self, rgb)
    }
}

#[derive(Debug, Clone, Copy, Hash, Display, PartialEq, Eq)]
pub enum LumaChannel {
    Variance,
    Z,
    RayDepth,
}
impl LumaChannel {
    pub fn channel<RgbStorage, LumaStorage>(
        self,
        luma: LumaStorage,
    ) -> Channel<RgbStorage, LumaStorage> {
        Channel::LumaChannel(self, luma)
    }
}

pub enum Channel<RgbStorage, LumaStorage> {
    RgbChannel(RgbChannel, RgbStorage),
    LumaChannel(LumaChannel, LumaStorage),
}

pub struct GenericRenderResult<RgbStorage, LumaStorage> {
    pub channels: Vec<Channel<RgbStorage, LumaStorage>>,
}
pub type PixelRenderResult = GenericRenderResult<Rgb, Luma>;

pub struct World<'a> {
    pub objects: &'a dyn Shape,
    pub lights: &'a [Point],
    pub materials: &'a [MaterialDescriptor],
    pub world_material: MaterialId,
}
