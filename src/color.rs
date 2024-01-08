use image::Rgb;

use crate::math::{
    self,
    vec::{RgbAsVec3Ext, Vec3, Vec3AsRgbExt},
};

pub type Color = Rgb<f64>;

pub const WHITE: Rgb<f64> = Rgb([1.0, 1.0, 1.0]);
pub const BLACK: Rgb<f64> = Rgb([0.0, 0.0, 0.0]);
pub const RED: Rgb<f64> = Rgb([1.0, 0.0, 0.0]);
pub const GREEN: Rgb<f64> = Rgb([0.0, 1.0, 0.0]);
pub const BLUE: Rgb<f64> = Rgb([0.0, 0.0, 1.0]);

pub enum MixMode {
    Add,
    Mul,
}

pub fn mix(mode: MixMode, color1: Color, color2: Color) -> Color {
    let vc1 = Vec3(color1.0);
    let vc2 = Vec3(color2.0);
    let vc_out = match mode {
        MixMode::Add => vc1 + vc2,
        MixMode::Mul => vc1 * vc2,
    };

    Rgb(vc_out.0)
}

pub fn lerp(t: f64, color1: Color, color2: Color) -> Color {
    Rgb(math::utils::lerp(t, Vec3(color1.0), Vec3(color2.0)).0)
}

pub fn clamp(color: Color) -> Color {
    math::utils::clamp(color.vec()).rgb()
}

pub fn gray(c: f64) -> Color {
    Rgb([c, c, c])
}

pub fn convert_lossy(c: Rgb<f64>) -> Rgb<f32> {
    Rgb(c.0.map(|x| x as f32))
}
