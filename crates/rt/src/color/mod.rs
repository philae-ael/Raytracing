use std::marker::PhantomData;

use bytemuck::{Pod, Zeroable};
use colorspace::Colorspace;

pub mod colorspace;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable)]
pub struct Color<S>(pub [f32; 3], PhantomData<S>)
where
    S: colorspace::Colorspace;

unsafe impl<S: colorspace::Colorspace> bytemuck::Pod for Color<S> {}

impl<S: colorspace::Colorspace> std::ops::Add for Color<S> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_array([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ])
    }
}
impl<S: colorspace::Colorspace> std::ops::Mul<Color<S>> for f32 {
    type Output = Color<S>;

    fn mul(self, rhs: Color<S>) -> Self::Output {
        Color::from_array([self * rhs.0[0], self * rhs.0[1], self * rhs.0[2]])
    }
}

impl<S: colorspace::Colorspace> std::ops::Div<f32> for Color<S> {
    type Output = Color<S>;

    fn div(self, rhs: f32) -> Self::Output {
        Color::from_array([self.0[0] / rhs, self.0[1] / rhs, self.0[2] / rhs])
    }
}

#[allow(non_camel_case_types)]
pub type sRgb = Color<colorspace::sRGB>;
pub type Rgb = Color<colorspace::Linear_RGB>;

impl<S: colorspace::Colorspace> Color<S> {
    pub const fn from_array(arr: [f32; 3]) -> Self {
        Self(arr, PhantomData)
    }

    pub const fn to_array(self) -> [f32; 3] {
        self.0
    }

    pub fn to_byte_array(self) -> [u8; 3] {
        self.0.map(|c| (c * 255. + 0.5) as u8)
    }
}

impl<S: colorspace::Colorspace> From<[f32; 3]> for Color<S> {
    fn from(val: [f32; 3]) -> Self {
        Color::<S>::from_array(val)
    }
}

pub trait ColorspaceConversion<C: colorspace::Colorspace> {
    fn convert(self) -> Color<C>;
}

impl<C: colorspace::Colorspace> ColorspaceConversion<C> for Color<C> {
    fn convert(self) -> Color<C> {
        self
    }
}

macro_rules! default_colorspace_conversions {
    ($c:ty) => {
        impl ColorspaceConversion<colorspace::CIE_XYZ> for Color<$c> {
            fn convert(self) -> Color<colorspace::CIE_XYZ> {
                Color::from_array(<$c>::to_cie_xyz(self.to_array()))
            }
        }
        impl ColorspaceConversion<$c> for Color<colorspace::CIE_XYZ> {
            fn convert(self) -> Color<$c> {
                Color::from_array(<$c>::from_cie_xyz(self.to_array()))
            }
        }
    };
}

// == sRGB ==
default_colorspace_conversions!(colorspace::sRGB);
impl ColorspaceConversion<colorspace::Linear_RGB> for sRgb {
    fn convert(self) -> Color<colorspace::Linear_RGB> {
        Color::from_array(self.to_array().map(colorspace::Linear_RGB::from_srgb))
    }
}

// == Linear RGB ==
default_colorspace_conversions!(colorspace::Linear_RGB);
impl ColorspaceConversion<colorspace::sRGB> for Rgb {
    fn convert(self) -> Color<colorspace::sRGB> {
        Color::from_array(self.to_array().map(colorspace::sRGB::from_linear_rgb))
    }
}

// == Luma ==
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct Luma(pub f32);

impl Luma {
    pub fn from_color<C: ColorspaceConversion<colorspace::CIE_XYZ>>(val: C) -> Self {
        Luma(val.convert().0[1])
    }
}

impl From<sRgb> for image::Rgb<f32> {
    fn from(val: sRgb) -> Self {
        image::Rgb(val.to_array())
    }
}

impl From<Luma> for image::Luma<f32> {
    fn from(val: Luma) -> Self {
        image::Luma([val.0])
    }
}

pub mod linear {
    use super::Rgb;

    pub const WHITE: Rgb = Rgb::from_array([1.0, 1.0, 1.0]);
    pub const BLACK: Rgb = Rgb::from_array([0.0, 0.0, 0.0]);
    pub const RED: Rgb = Rgb::from_array([1.0, 0.0, 0.0]);
    pub const GREEN: Rgb = Rgb::from_array([0.0, 1.0, 0.0]);
    pub const BLUE: Rgb = Rgb::from_array([0.0, 0.0, 1.0]);
}
