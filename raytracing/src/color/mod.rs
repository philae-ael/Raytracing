use std::marker::PhantomData;

pub mod colorspace;

#[derive(Clone, Copy, Debug)]
pub struct Color<S: colorspace::Colorspace>(pub [f32; 3], PhantomData<S>);

impl<S: colorspace::Colorspace> Color<S> {
    pub const fn from_array(arr: [f32; 3]) -> Self {
        Self(arr, PhantomData)
    }

    pub const fn to_array(self) -> [f32; 3] {
        self.0
    }

    pub fn convert<S2: colorspace::Colorspace>(self) -> Color<S2> {
        Color::from_array(S2::from_cie_xyz(S::to_cie_xyz(self.to_array())))
    }
}

impl<S: colorspace::Colorspace> Into<Color<S>> for [f32; 3] {
    fn into(self) -> Color<S> {
        Color::<S>::from_array(self)
    }
}

#[allow(non_camel_case_types)]
pub type sRgb = Color<colorspace::sRGB>;
impl sRgb {
    pub fn to_rgb(self) -> Rgb {
        self.to_array()
            .map(colorspace::Linear_sRGB::from_srgb)
            .into()
    }
}
pub type Rgb = Color<colorspace::Linear_sRGB>;
impl Rgb {
    pub fn to_srgb(self) -> sRgb {
        self.to_array()
            .map(colorspace::sRGB::from_linear_rgb)
            .into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Luma(pub f32);

impl<S: colorspace::Colorspace> Into<Luma> for Color<S> {
    fn into(self) -> Luma {
        Luma(self.convert::<colorspace::CIE_XYZ>().0[1])
    }
}

impl Into<image::Rgb<f32>> for sRgb {
    fn into(self) -> image::Rgb<f32> {
        image::Rgb(self.to_array())
    }
}

impl Into<image::Luma<f32>> for Luma {
    fn into(self) -> image::Luma<f32> {
        image::Luma([self.0])
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

pub enum MixMode {
    Add,
    Mul,
}
