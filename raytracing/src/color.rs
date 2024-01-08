use image::Rgb;

use crate::math::vec::{RgbAsVec3Ext, Vec3, Vec3AsRgbExt};

pub type Color = Rgb<f32>;

pub const WHITE: Color = Rgb([1.0, 1.0, 1.0]);
pub const BLACK: Color = Rgb([0.0, 0.0, 0.0]);
pub const RED: Color = Rgb([1.0, 0.0, 0.0]);
pub const GREEN: Color = Rgb([0.0, 1.0, 0.0]);
pub const BLUE: Color = Rgb([0.0, 0.0, 1.0]);

pub enum MixMode {
    Add,
    Mul,
}

pub fn mix(mode: MixMode, color1: Color, color2: Color) -> Color {
    let vc1 = color1.vec();
    let vc2 = color2.vec();
    let vc_out = match mode {
        MixMode::Add => vc1 + vc2,
        MixMode::Mul => vc1 * vc2,
    };

    vc_out.rgb()
}

pub fn lerp(t: f32, color1: Color, color2: Color) -> Color {
    color1.vec().lerp(color2.vec(), t).rgb()
}

pub fn clamp(color: Color) -> Color {
    color.vec().clamp(Vec3::ZERO, Vec3::ONE).rgb()
}

pub fn gray(c: f32) -> Color {
    Rgb([c, c, c])
}
