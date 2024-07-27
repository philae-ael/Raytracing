use bytemuck::{Pod, Zeroable};

use crate::{
    color::{self, Luma, Rgb},
    material::{MaterialDescriptor, MaterialId},
    math::{
        point::Point,
        stat::RgbSeries,
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

pub struct RaySeries {
    pub normal: Vec3,
    pub position: Point,
    pub albedo: Rgb,
    pub color: RgbSeries,
    pub z: f32,
    pub ray_depth: f32,
    pub samples_accumulated: u32,
}

impl RaySeries {
    pub fn as_pixelresult(&self) -> PixelRenderResult {
        let RaySeries {
            position,
            normal,
            albedo,
            color,
            z,
            ray_depth,
            samples_accumulated,
        } = self;

        let inv_samples = 1.0 / *samples_accumulated as f32;
        PixelRenderResult {
            normal: (inv_samples * *normal).rgb(),
            position: (inv_samples * position.vec()).rgb(),
            albedo: (inv_samples * albedo.vec()).rgb(),
            color: color.mean(),
            z: color::Luma(inv_samples * z),
            ray_depth: color::Luma(inv_samples * ray_depth),
        }
    }

    pub fn add_sample(&mut self, rhs: RayResult) {
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

impl Default for RaySeries {
    fn default() -> Self {
        Self {
            normal: color::linear::BLACK.vec(),
            position: Point::ORIGIN,
            albedo: color::linear::BLACK,
            color: RgbSeries::default(),
            z: 0.0,
            ray_depth: 0.0,
            samples_accumulated: 0,
        }
    }
}

pub enum Channel<RgbStorage, LumaStorage> {
    Color(RgbStorage),
    Normal(RgbStorage),
    Position(RgbStorage),
    Albedo(RgbStorage),
    Z(LumaStorage),
    RayDepth(LumaStorage),
}
const CHANNEL_COUNT: usize = 6;

#[repr(C)]
pub struct GenericRenderResult<RgbStorage, LumaStorage> {
    pub color: RgbStorage,
    pub normal: RgbStorage,
    pub position: RgbStorage,
    pub albedo: RgbStorage,
    pub z: LumaStorage,
    pub ray_depth: LumaStorage,
}

impl<RgbStorage, LumaStorage> GenericRenderResult<RgbStorage, LumaStorage> {
    pub fn as_ref(&self) -> GenericRenderResult<&RgbStorage, &LumaStorage> {
        GenericRenderResult {
            color: &self.color,
            normal: &self.normal,
            position: &self.position,
            albedo: &self.albedo,
            z: &self.z,
            ray_depth: &self.ray_depth,
        }
    }
}

impl<T: Copy, L: Copy> Copy for GenericRenderResult<T, L> {}
impl<T: Clone, L: Clone> Clone for GenericRenderResult<T, L> {
    fn clone(&self) -> Self {
        Self {
            color: self.color.clone(),
            position: self.position.clone(),
            normal: self.normal.clone(),
            albedo: self.albedo.clone(),
            z: self.z.clone(),
            ray_depth: self.ray_depth.clone(),
        }
    }
}

pub type PixelRenderResult = GenericRenderResult<Rgb, Luma>;

impl<RgbStorage, LumaStorage> IntoIterator for GenericRenderResult<RgbStorage, LumaStorage> {
    type Item = Channel<RgbStorage, LumaStorage>;

    type IntoIter = <[Channel<RgbStorage, LumaStorage>; CHANNEL_COUNT] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        [
            Channel::Color(self.color),
            Channel::Albedo(self.albedo),
            Channel::Position(self.position),
            Channel::Normal(self.normal),
            Channel::Z(self.z),
            Channel::RayDepth(self.ray_depth),
        ]
        .into_iter()
    }
}

/// SAFETY:
/// - GenericRenderResult is Zeroable as T and L are,
/// - all bits patterns are valid as all are valid for T and L,
/// - all his fields are pods,
/// - it is repr(C),
/// - there is no interior mutability
unsafe impl<T: Pod, L: Pod> Pod for GenericRenderResult<T, L> {}
///
/// SAFETY:
/// GenericRenderResult is inhabited and the all-zero pattern is allowed as they are valid for T and L
unsafe impl<T: Zeroable, L: Zeroable> Zeroable for GenericRenderResult<T, L> {}

pub struct World<'a> {
    pub objects: &'a dyn Shape,
    pub lights: &'a [Point],
    pub materials: &'a [MaterialDescriptor],
    pub world_material: MaterialId,
}
