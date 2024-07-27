/// The colorspaces
///
/// Color don't exist. Yet we want to represent them and store them
/// Thus the human created colorspace. A mathematical representation of colors.
///
/// Facts: Human have 3 cones + perception is kinda linear
/// => We can use 3 numbers to identify a colors, in a linear space
///
///
/// Source: Wikipedia, this good youtube video: https://youtu.be/AS1OHMW873s
use glam::{Mat3, Vec3};

pub trait Colorspace: Copy + Clone + Send + Sync + bytemuck::Zeroable + bytemuck::Pod {
    fn from_cie_xyz(coord: [f32; 3]) -> [f32; 3];
    fn to_cie_xyz(coord: [f32; 3]) -> [f32; 3];
}

/// Linear sRGB: cover only a fraction of CIE XYZ. But yet good enough. Still linear, and has a basis composed of the primary colors: R,G and B
///
/// The way to go for math manipulations and easy realistic usages
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
#[allow(non_camel_case_types)]
pub struct Linear_RGB;
impl Linear_RGB {
    pub fn from_srgb(srgb: f32) -> f32 {
        let srgb = srgb.clamp(0.0, 1.0);
        if srgb.is_nan() {
            0.0
        } else if srgb <= 0.04045 {
            srgb / 12.92
        } else {
            ((srgb + 0.055) / 1.055).powf(2.4)
        }
    }
}

impl Colorspace for Linear_RGB {
    fn from_cie_xyz(coord: [f32; 3]) -> [f32; 3] {
        let mat = Mat3::from_cols(
            Vec3::new(3.2406, -0.9689, 0.0557),
            Vec3::new(-1.5372, 1.8758, -0.2040),
            Vec3::new(-0.4986, 0.0415, 1.0570),
        );

        (mat * Vec3::from_array(coord)).to_array()
    }

    fn to_cie_xyz(coord: [f32; 3]) -> [f32; 3] {
        let mat = Mat3::from_cols(
            Vec3::new(0.4124, 0.2126, 0.0193),
            Vec3::new(0.3576, 0.7152, 0.1192),
            Vec3::new(0.1805, 0.0722, 0.9505),
        );

        (mat * Vec3::from_array(coord)).to_array()
    }
}

/// sRGB same as Linear sRGB but transformed with a non linear transform such that low brightness values (which differences are perceptually easier to distinguish)
/// have more space to work with (mainly when using integers for colors)
///
/// YOU CAN'T DO MATH ON sRGB USE LINEAR sRGB OR XYZ
///
/// The way to go for storage and sending data to displays. It's a transfert format
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
#[allow(non_camel_case_types)]
pub struct sRGB;
impl sRGB {
    pub fn from_linear_rgb(linear: f32) -> f32 {
        let linear = linear.clamp(0.0, 1.0);
        if linear.is_nan() {
            0.0
        } else if linear < 0.0031308 {
            12.92 * linear
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        }
    }
}
impl Colorspace for sRGB {
    fn from_cie_xyz(coord: [f32; 3]) -> [f32; 3] {
        Linear_RGB::from_cie_xyz(coord).map(Self::from_linear_rgb)
    }

    fn to_cie_xyz(coord: [f32; 3]) -> [f32; 3] {
        Linear_RGB::from_cie_xyz(coord.map(Linear_RGB::from_srgb))
    }
}

/// Cover all the visible colors and more, in a linear space (useful values are in [0, 1]^3)
/// CIE XYZ is the reference frame for colorspaces. All colorspaces can be obtained by a transformation of XYZ
/// Its basis as a vector space is composed of non colors.
/// Y is the relative luminance of the color (relative to the display max brightness)
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
#[allow(non_camel_case_types)]
pub struct CIE_XYZ;
impl Colorspace for CIE_XYZ {
    fn from_cie_xyz(coord: [f32; 3]) -> [f32; 3] {
        coord
    }

    fn to_cie_xyz(coord: [f32; 3]) -> [f32; 3] {
        coord
    }
}
